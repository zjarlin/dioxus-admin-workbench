# Admin Shell Core

`az-admin-shell-core` 定义后台工作台的正式协议：场景、菜单、页面、资源、变更命令，以及页面扩展的校验与编译 SPI。它不依赖 Dioxus，不保存任何渲染阶段对象。

```toml
[dependencies]
az-admin-shell-core = "2026.8.11"
```

页面来源只有两类：消费方约定文件，或通过 Rudi 注册的扩展。CRUD 是独立扩展，不属于核心枚举。
