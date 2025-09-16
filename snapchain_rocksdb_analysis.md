# Snapchain RocksDB Storage Architecture Analysis

## Overview

Snapchain uses RocksDB as its underlying storage engine, employing a sharded architecture to store Farcaster protocol data. This document provides a detailed analysis of Snapchain's RocksDB storage implementation.

## Core Architecture

### 1. Database Sharding Structure

Snapchain uses multiple independent RocksDB instances to store different types of data:

```
rocksdb_dir/
├── shard-0/          # Block storage (Block Shard)
├── shard-1/          # Shard 1 data
├── shard-2/          # Shard 2 data
├── shard-N/          # Shard N data
└── global/           # Global data
```

### 2. Storage Optimization Types

Snapchain supports two database optimization modes:

- **Default**: Default read-write optimization
- **BulkWriteOptimized**: High-volume write optimization
  - Write buffer: 512MB
  - Parallelism: 8 background threads
  - Max write buffers: 4
  - Min merge buffers: 2

## Key-Value Storage Patterns

### 1. Root Prefix (RootPrefix)

```rust
pub enum RootPrefix {
    Block = 1,                    // Block data
    Shard = 2,                    // Shard data
    User = 3,                     // User data (starts with 4-byte FID)
    CastsByParent = 4,            // Casts indexed by parent
    CastsByMention = 5,           // Casts indexed by mention
    LinksByTarget = 6,            // Links indexed by target
    ReactionsByTarget = 7,        // Reactions indexed by target
    MerkleTrieNode = 8,           // Merkle Trie nodes
    HubEvents = 9,                // Event logs
    FNameUserNameProof = 11,      // FName username proofs
    OnChainEvent = 12,            // On-chain events
    DBSchemaVersion = 13,         // Database schema version
    VerificationByAddress = 14,   // Verifications indexed by address
    FNameUserNameProofByFid = 15, // FName proofs indexed by FID
    UserNameProofByName = 16,     // Username proofs indexed by name
    NodeLocalState = 17,          // Node local state
    BlockIndex = 18,              // Block timestamp index
    BlockEvent = 19,              // Block event sequence number index
    MerkleTrieMetadata = 20,      // Merkle Trie metadata
    ReplicationBootstrapStatus = 21, // Replication bootstrap status
}
```

### 2. User Data Postfix (UserPostfix)

```rust
pub enum UserPostfix {
    // Message records (1-85)
    CastMessage = 1,
    LinkMessage = 2,
    ReactionMessage = 3,
    VerificationMessage = 4,
    UserDataMessage = 6,
    UsernameProofMessage = 7,
    
    // Index records (86-255)
    CastAdds = 87,
    CastRemoves = 88,
    LinkAdds = 89,
    LinkRemoves = 90,
    ReactionAdds = 91,
    ReactionRemoves = 92,
    VerificationAdds = 93,
    VerificationRemoves = 94,
    UserDataAdds = 97,
    UserNameProofAdds = 99,
    LinkCompactStateMessage = 100,
}
```

## 键值生成策略

### 1. 主键生成

```rust
// 用户主键: [RootPrefix::User] + [FID(4字节)]
fn make_user_key(fid: u64) -> Vec<u8> {
    let mut key = Vec::with_capacity(1 + 4);
    key.push(RootPrefix::User as u8);
    key.extend_from_slice(&make_fid_key(fid));
    key
}

// 消息主键: [UserKey] + [SetPostfix] + [TSHash(20字节)]
fn make_message_primary_key(fid: u64, set: u8, ts_hash: Option<&[u8; 20]>) -> Vec<u8> {
    let mut key = Vec::with_capacity(1 + 4 + 1 + 20);
    key.extend_from_slice(&make_user_key(fid));
    key.push(set);
    if ts_hash.is_some() {
        key.extend_from_slice(ts_hash.unwrap());
    }
    key
}
```

### 2. 时间戳哈希 (TSHash)

```rust
// TSHash = [Timestamp(4字节)] + [Hash(20字节)]
fn make_ts_hash(timestamp: u32, hash: &[u8; 20]) -> Result<[u8; 24], HubError> {
    let mut ts_hash = [0u8; 24];
    ts_hash[0..4].copy_from_slice(&timestamp.to_be_bytes());
    ts_hash[4..24].copy_from_slice(hash);
    Ok(ts_hash)
}
```

### 3. 区块键生成

```rust
// 区块主键: [RootPrefix::Block] + [BlockNumber(8字节)]
fn make_block_key(block_number: u64) -> Vec<u8> {
    let mut key = vec![RootPrefix::Block as u8];
    key.extend_from_slice(&block_number.to_be_bytes());
    key
}

// 分片键: [RootPrefix::Shard] + [BlockNumber(8字节)]
fn make_shard_key(block_number: u64) -> Vec<u8> {
    let mut key = vec![RootPrefix::Shard as u8];
    key.extend_from_slice(&block_number.to_be_bytes());
    key
}
```

## 存储结构

### 1. 事务批处理

```rust
pub struct RocksDbTransactionBatch {
    pub batch: HashMap<Vec<u8>, Option<Vec<u8>>>,
}

impl RocksDbTransactionBatch {
    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.batch.insert(key, Some(value));
    }
    
    pub fn delete(&mut self, key: Vec<u8>) {
        self.batch.insert(key, None);
    }
}
```

### 2. 数据编码

- **消息编码**: 使用 Protocol Buffers 序列化
- **压缩**: LZ4 压缩算法
- **事务支持**: 使用 TransactionDB 支持 ACID 事务

## 分片机制

### 1. 分片分配

- **分片0**: 保留给区块存储
- **分片1-N**: 用户数据分片
- **全局数据库**: 跨分片共享数据

### 2. 分片存储

```rust
// 为每个分片创建独立的RocksDB实例
pub fn open_shard_db(db_dir: &str, shard_id: u32) -> Arc<RocksDB> {
    let db = RocksDB::new(format!("{}/shard-{}", db_dir, shard_id).as_str());
    db.open().unwrap();
    Arc::new(db)
}
```

## 存储类型

### 1. 消息存储

- **Cast消息**: 用户发布的文本内容
- **Link消息**: 用户之间的关注关系
- **Reaction消息**: 对消息的反应
- **Verification消息**: 地址验证
- **UserData消息**: 用户资料数据
- **UsernameProof消息**: 用户名证明

### 2. 用户数据存储详解

#### 用户数据类型 (UserDataType)

```rust
pub enum UserDataType {
    USER_DATA_TYPE_NONE = 0,
    USER_DATA_TYPE_PFP = 1,                    // 用户头像
    USER_DATA_TYPE_DISPLAY = 2,                // 显示名称
    USER_DATA_TYPE_BIO = 3,                    // 个人简介
    USER_DATA_TYPE_URL = 5,                    // 个人网站
    USER_DATA_TYPE_USERNAME = 6,               // 用户名
    USER_DATA_TYPE_LOCATION = 7,               // 位置信息
    USER_DATA_TYPE_TWITTER = 8,                // Twitter用户名
    USER_DATA_TYPE_GITHUB = 9,                 // GitHub用户名
    USER_DATA_TYPE_BANNER = 10,                // 横幅图片
    USER_DATA_PRIMARY_ADDRESS_ETHEREUM = 11,   // 以太坊主地址
    USER_DATA_PRIMARY_ADDRESS_SOLANA = 12,     // Solana主地址
    USER_DATA_TYPE_PROFILE_TOKEN = 13,         // 个人资料代币 (CAIP-19格式)
}
```

#### 用户数据存储结构

**主键结构**:
```
[RootPrefix::User] + [FID(4字节)] + [UserPostfix::UserDataAdds] + [UserDataType]
```

**存储键生成**:
```rust
fn make_user_data_adds_key(fid: u64, data_type: i32) -> Vec<u8> {
    let mut key = Vec::with_capacity(33 + 1 + 1);
    key.extend_from_slice(&make_user_key(fid));           // [RootPrefix::User] + [FID]
    key.push(UserPostfix::UserDataAdds as u8);            // 97
    if data_type > 0 {
        key.push(data_type as u8);                        // UserDataType
    }
    key
}
```

#### 用户数据查询方法

1. **按FID和类型查询**:
   ```rust
   pub fn get_user_data_by_fid_and_type(
       store: &Store<UserDataStoreDef>,
       fid: u64,
       user_data_type: proto::UserDataType,
   ) -> Result<proto::Message, HubError>
   ```

2. **按FID查询所有用户数据**:
   ```rust
   pub fn get_user_data_adds_by_fid(
       store: &Store<UserDataStoreDef>,
       fid: u64,
       page_options: &PageOptions,
       start_time: Option<u32>,
       stop_time: Option<u32>,
   ) -> Result<MessagesPage, HubError>
   ```

3. **按类型查询特定用户数据**:
   ```rust
   pub fn get_user_data_add(
       store: &Store<UserDataStoreDef>,
       fid: u64,
       r#type: i32,
   ) -> Result<Option<proto::Message>, HubError>
   ```

#### 用户数据存储特点

- **只支持添加**: 用户数据不支持删除操作，只能更新
- **类型唯一性**: 每种用户数据类型只能有一个值
- **时间戳排序**: 按时间戳排序，最新的值有效
- **分页支持**: 支持分页查询大量用户数据
- **时间范围查询**: 支持按时间范围过滤用户数据

#### 用户数据存储示例

**存储键示例**:
```
// FID 12345 的用户名数据
[RootPrefix::User(3)] + [FID(12345)] + [UserDataAdds(97)] + [Username(6)]
= [3, 0, 0, 48, 57, 97, 6]

// FID 12345 的头像数据  
[RootPrefix::User(3)] + [FID(12345)] + [UserDataAdds(97)] + [PFP(1)]
= [3, 0, 0, 48, 57, 97, 1]
```

**存储值示例**:
```protobuf
message UserDataBody {
  UserDataType type = 1;        // 数据类型 (如 USER_DATA_TYPE_USERNAME)
  string value = 2;             // 数据值 (如 "alice")
}

message Message {
  MessageData data = 1;         // 包含 UserDataBody
  bytes hash = 2;               // 消息哈希
  bytes signature = 4;          // 签名
  // ... 其他字段
}
```

#### 用户数据查询优先级

在消息处理时，用户数据有特定的处理优先级：

```rust
fn get_message_priority(msg: &proto::Message) -> u8 {
    match msg.msg_type() {
        MessageType::VerificationAddEthAddress => 1,    // 最高优先级
        MessageType::UsernameProof => 2,                // 用户名证明
        MessageType::UserDataAdd => {
            if let Some(Body::UserDataBody(body)) = &msg.data.as_ref().unwrap().body {
                let user_data_type = body.r#type();
                if user_data_type == UserDataType::Username
                    || user_data_type == UserDataType::UserDataPrimaryAddressEthereum
                    || user_data_type == UserDataType::UserDataPrimaryAddressSolana
                {
                    return 3; // 依赖的用户数据类型
                }
            }
            4 // 其他用户数据类型
        }
        _ => 5, // 其他消息类型
    }
}
```

#### 用户名证明存储

除了用户数据，Snapchain还存储用户名证明：

```rust
// 用户名证明存储键
[RootPrefix::FNameUserNameProof(11)] + [FID(4字节)] + [Name(32字节)]

// 按FID索引的用户名证明
[RootPrefix::FNameUserNameProofByFid(15)] + [FID(4字节)] + [Name(32字节)]

// 按名称索引的用户名证明  
[RootPrefix::UserNameProofByName(16)] + [Name(32字节)] + [FID(4字节)]
```

### 2. 索引存储

- **二级索引**: 支持按不同维度查询
- **时间戳索引**: 支持时间范围查询
- **关系索引**: 支持关系查询

### 3. 状态存储

- **Merkle Trie**: 用于状态验证
- **节点状态**: 本地节点状态信息
- **复制状态**: 数据复制进度

## 性能优化

### 1. 写入优化

- **批量写入**: 使用事务批处理
- **并行写入**: 多线程后台处理
- **缓冲区管理**: 优化的写缓冲区大小

### 2. 读取优化

- **分页查询**: 支持分页结果
- **索引查询**: 利用二级索引加速查询
- **缓存机制**: 内存缓存热点数据

### 3. 压缩优化

- **LZ4压缩**: 减少存储空间
- **数据去重**: 避免重复存储
- **定期压缩**: 后台压缩优化

## 数据一致性

### 1. 事务支持

- **ACID属性**: 保证数据一致性
- **并发控制**: 读写锁机制
- **回滚支持**: 事务失败时回滚

### 2. 复制机制

- **数据复制**: 跨节点数据同步
- **一致性检查**: 定期验证数据一致性
- **故障恢复**: 自动故障检测和恢复

## 总结

Snapchain 的 RocksDB 存储架构具有以下特点：

1. **分片设计**: 通过分片实现水平扩展
2. **键值优化**: 精心设计的键值结构支持高效查询
3. **事务支持**: 完整的 ACID 事务保证
4. **性能优化**: 多种优化策略提升性能
5. **可扩展性**: 支持大规模数据存储和查询

这种架构使得 Snapchain 能够高效地存储和查询 Farcaster 协议的海量数据，同时保持良好的性能和一致性。
