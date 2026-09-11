# Dioxus Admin Shell

`az-dioxus-admin-shell` 同时提供无业务依赖的发布应用壳和由单一 `AdminProvider` 驱动的元数据工作台。

`ApplicationShell` 负责可折叠侧栏、移动导航、页面工具栏、菜单树和账户菜单。消费方只传入菜单模型、当前页面内容及操作回调；壳层不认识业务菜单、接口地址、持久化模型或生成产物。

`ApplicationFullscreenPage` 提供独立全屏页面容器，消费 `application_label`、`page_label`、`on_back` 和页面内容。账户菜单打开的功能在此容器中展示；顶部只保留返回主后台操作和当前页面标题，不显示场景或侧栏。调用方保存原后台导航状态，并在返回回调中恢复。

可安装页面仓库实现 `ApplicationPlugin`，由 Dill 绑定具体类型。`collect_application_pages` 按 `TypeId` 拒绝重复插件，并校验页面和场景导航；`PluginApplication` 将结果直接编排进同一个 `ApplicationShell`。插件不声明字符串运行时身份，页面 `id` 只用于业务导航。

场景是菜单树的根，由顶部标签切换；侧栏只渲染当前根的后代，不把场景再包装成侧栏分组。页面通过 `menu_path` 显式声明从场景根到叶子页面之间的分组路径，空路径表示直属页面。多个插件可贡献相同 `id`、场景、父节点及展示信息的分组，壳会把这些页面合并到一个节点；标题、图标、场景或父节点不一致会拒绝整个组合，路径内重复分组同样按循环拒绝。

```rust,no_run
use az_dioxus_admin_shell::{ApplicationMenuGroup, ApplicationPage, ApplicationScene};
use dioxus::prelude::*;

fn dictionary_page() -> Element {
    rsx! { p { "字典管理" } }
}

let page = ApplicationPage {
    id: "dictionary",
    label: "字典管理",
    icon: Some("book_open"),
    scene: ApplicationScene { id: "system", label: "系统" },
    menu_path: vec![ApplicationMenuGroup {
        id: "system-management".to_owned(),
        label: "系统管理".to_owned(),
        icon: Some("settings".to_owned()),
    }],
    required_permission: Some("dictionary:read"),
    render: dictionary_page,
};
```

`ApplicationMenuGroup` 同时用于编译进壳的 `ApplicationPage` 和动态 `ApplicationRuntimePage`，因此二进制插件与运行时插件遵循同一棵树契约。分组节点可逐层展开、折叠；切换顶部场景时会进入该根下的第一个后代页面，而不是假设第一层节点就是页面。

`ApplicationAccountPlugin` 通过 `page_id` 贡献的页面属于账户全屏入口，不进入场景菜单树。此规则同样用于运行时插件及子插件，壳不根据页面名称判断。打开账户页面时后台组件保持挂载，返回后恢复原场景、页面和页面内部状态；退出等无 `page_id` 动作继续交给宿主回调。

`PluginApplication` 懒挂载页面并保留有限实例，默认 `workspace_cache_capacity = 6`、`account_cache_capacity = 2`。切换页面只改变可见性，保持组件、iframe 和 DOM 插入顺序；超过容量淘汰非当前的 LRU 实例。消费方通过 `runtime_page_versions` 提供每页版本或激活代次，描述改变、页面移除或版本改变会销毁旧实例。切换用户/租户时消费方必须重建应用根，不能跨上下文复用页面池。页面容器的 `data-aio-page-active` 标记供宿主向隔离前端通知显隐，壳不读取或管理插件内部状态。

不需要元数据工作台时关闭默认 feature，依赖中不会包含 Provider 注册运行时：

```toml
[dependencies]
az-dioxus-admin-shell = { version = "2026.8.17", default-features = false }
```

需要场景、菜单和页面元数据编辑能力时启用默认的 `workbench` feature，并由消费方实现、注册一个 `AdminProvider`，然后启动 `az_dioxus_admin_shell::App`。CRUD 由 `az-dioxus-admin-extension-crud` 独立提供，完整示例位于仓库 `examples/web`。
