# oxpm_npm

`oxpm_npm` 是 oxpm 对 npm 生态的兼容层，用于处理 npm 相关的数据结构、协议兼容和行为适配

仅支持 npm v9+ 的 npm 生态

## 目标

- 解析和生成 npm 生态中的核心文件格式
- 兼容 npm registry 返回的数据结构
- 为 oxpm 的安装、解析、锁定流程提供 npm 兼容能力
- 尽量保持与 npm 行为一致，同时为 oxpm 内部模型提供稳定接口

## 路线图

- [x] `package-lock.json` 兼容
  - [x] 支持 `lockfileVersion` v1 / v2 / v3
  - [x] 按版本分发到对应的数据结构
  - [x] 支持 lockfile 解析和序列化