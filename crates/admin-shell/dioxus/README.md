# Dioxus Admin Shell

`az-dioxus-admin-shell` 同时提供无业务依赖的发布应用壳和由单一 `AdminProvider` 驱动的元数据工作台。

`ApplicationShell` 负责可折叠侧栏、移动导航、页面工具栏、菜单树和账户菜单。消费方只传入菜单模型、当前页面内容及操作回调；壳层不认识业务菜单、接口地址、持久化模型或生成产物。

`ApplicationFullscreenPage` 提供独立全屏页面容器，消费 `application_label`、`page_label`、`on_back` 和页面内容。账户菜单打开的功能在此容器中展示；顶部只保留返回主后台操作和当前页面标题，不显示场景或侧栏。调用方保存原后台导航状态，并在返回回调中恢复。

可安装页面仓库实现 `ApplicationPlugin`，由 Dill 绑定具体类型。`collect_application_pages` 按 `TypeId` 拒绝重复插件，并校验页面和场景导航；`PluginApplication` 将结果直接编排进同一个 `ApplicationShell`。插件不声明字符串运行时身份，页面 `id` 只用于业务导航。

场景是菜单树的根，由顶部标签切换；侧栏只渲染当前根的后代，不把场景再包装成侧栏分组。`ApplicationAccountPlugin` 通过 `page_id` 贡献的页面属于账户全屏入口，不进入场景菜单树。此规则同样用于运行时插件及子插件，壳不根据页面名称判断。打开账户页面时后台组件保持挂载，返回后恢复原场景、页面和页面内部状态；退出等无 `page_id` 动作继续交给宿主回调。

不需要元数据工作台时关闭默认 feature，依赖中不会包含 Provider 注册运行时：

```toml
[dependencies]
az-dioxus-admin-shell = { version = "2026.8.17", default-features = false }
```

需要场景、菜单和页面元数据编辑能力时启用默认的 `workbench` feature，并由消费方实现、注册一个 `AdminProvider`，然后启动 `az_dioxus_admin_shell::App`。CRUD 由 `az-dioxus-admin-extension-crud` 独立提供，完整示例位于仓库 `examples/web`。
