# Dioxus Admin Shell

`az-dioxus-admin-shell` 同时提供无业务依赖的发布应用壳和由单一 `AdminProvider` 驱动的元数据工作台。

`ApplicationShell` 负责可折叠侧栏、移动导航、页面工具栏、菜单树和账户菜单。消费方只传入菜单模型、当前页面内容及操作回调；壳层不认识业务菜单、接口地址、持久化模型或生成产物。

不需要元数据工作台时关闭默认 feature，依赖中不会包含 Provider 注册运行时：

```toml
[dependencies]
az-dioxus-admin-shell = { version = "2026.8.17", default-features = false }
```

需要场景、菜单和页面元数据编辑能力时启用默认的 `workbench` feature，并由消费方实现、注册一个 `AdminProvider`，然后启动 `az_dioxus_admin_shell::App`。CRUD 由 `az-dioxus-admin-extension-crud` 独立提供，完整示例位于仓库 `examples/web`。
