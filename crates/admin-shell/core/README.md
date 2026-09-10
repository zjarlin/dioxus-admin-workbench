# Admin Shell Core

`az-admin-shell-core` 定义后台工作台的正式协议：场景、菜单、页面、资源、变更命令，以及页面扩展的校验与编译 SPI。它不依赖 Dioxus，不保存任何渲染阶段对象。

```toml
[dependencies]
az-admin-shell-core = "2026.8.13"
```

页面来源只有两类：消费方约定文件，或通过 Rudi 注册的扩展。CRUD 是独立扩展，不属于核心枚举。

`SceneDefinition` 是一棵菜单树的根；`MenuDefinition.children` 保存任意深度的分组或页面节点。场景由顶部入口切换，渲染侧栏时只展示当前场景的 `menus`，不得再次把场景标题包装为侧栏分组。
