# Farcaster 客户端识别机制

## 🔍 概述

在 Farcaster 上看到的"消息来自某个客户端"的标识，实际上是通过多种方式实现的，而**不是协议层面的强制要求**。

## 📋 实现方式

### 1. **协议层面**
Farcaster 协议本身**不包含强制的客户端字段**，但提供了几种可选方式：

```protobuf
message CastAddBody {
  repeated Embed embeds = 6;  // 可以包含客户端标识
  CastType type = 8;          // Cast 类型
}
```

### 2. **实际识别方法**

#### 方法 A: 通过 Signer 映射
每个客户端应用通常会注册自己的 signer：
- Warpcast 有自己的 signer pool
- Supercast 有自己的 signer pool
- 第三方应用有独立的 signer

**Hub 或前端应用维护 signer → 客户端的映射表**

#### 方法 B: 通过 Embeds (自定义)
某些客户端可能在 embeds 中添加标识：
```json
{
  "embeds": [
    {
      "url": "https://warpcast.com/~/compose"
    }
  ]
}
```

#### 方法 C: 通过前端推断
前端应用（如 Warpcast）可以：
1. 查询 cast 的 signer
2. 查找该 signer 注册时的元数据
3. 从元数据中提取客户端信息

### 3. **Neynar Hub API 的情况**

当前 Neynar Hub API 返回的是**原始 Farcaster 消息格式**：
```json
{
  "messages": [
    {
      "data": { ... },
      "hash": "0x...",
      "signer": "0x..."  // Ed25519 公钥
    }
  ]
}
```

**没有直接的 `client` 或 `via` 字段**

## 🎯 如何在 Warpcast 看到客户端信息？

Warpcast 使用以下策略：

1. **自己的 casts** - 直接知道是从 Warpcast 发的
2. **其他客户端的 casts** - 通过以下方式识别：
   - 维护已知客户端的 signer 列表
   - 查询 signer 的注册元数据
   - 从特定的 embed 模式识别
   - 从 Frame actions 识别

## 📊 我们的实现

### 当前功能
```bash
castorix hub casts <FID> --limit N
```

显示信息：
- ⏰ 时间戳
- 🔗 Hash
- 🔑 Signer (Ed25519 公钥)
- 📝 文本内容
- 📎 Embeds
- 👥 Mentions

### 未来可能的增强

1. **Signer 数据库** - 维护常见客户端的 signer 映射
2. **启发式检测** - 通过 embed 模式推断客户端
3. **API 增强** - 如果 Neynar 添加客户端字段，自动显示

## 🔧 技术细节

### Cast 消息结构 (Protocol Buffers)

```protobuf
message Message {
  MessageData data = 1;
  bytes hash = 2;
  bytes signer = 6;  // Ed25519 公钥，是客户端识别的关键
}

message MessageData {
  uint64 fid = 2;
  uint32 timestamp = 3;
  CastAddBody cast_add_body = 5;
}

message CastAddBody {
  string text = 4;
  repeated Embed embeds = 6;
  repeated uint64 mentions = 2;
}
```

### 为什么没有强制的客户端字段？

1. **去中心化原则** - Farcaster 是去中心化协议
2. **灵活性** - 不限制客户端如何标识自己
3. **隐私** - 用户可以选择不暴露使用的客户端
4. **简洁性** - 保持核心协议简单

## 💡 总结

Farcaster 的客户端识别是：
- ✅ 应用层功能，不是协议要求
- ✅ 主要通过 **Signer 映射** 实现
- ✅ 不同的前端应用有各自的实现方式
- ❌ 不是 Hub API 返回的直接字段

如果你想实现类似功能，需要：
1. 维护一个 signer → 客户端的映射数据库
2. 或者使用支持客户端标识的更高级 API（如 Neynar v2 API）
