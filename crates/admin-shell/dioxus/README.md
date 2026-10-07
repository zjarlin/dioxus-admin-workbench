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

`PluginApplication` 懒挂载页面并保留有限实例，默认 `workspace_cache_capacity = 6`、`account_cache_capacity = 2`，容量跨所有工作区计算。切换页面只改变可见性，保持组件、iframe 和 DOM 插入顺序；超过容量淘汰非当前的 LRU 实例。消费方通过 `runtime_page_versions` 提供每页版本或激活代次，描述改变、页面移除或版本改变会销毁旧实例。

登录会话变化时消费方必须重建应用根；同一登录会话的租户/工作区切换则传入 `workspace_id` 和 `workspace_context`（租户、用户及权限摘要），不重建根。运行时页面按工作区分别保存实例与当前导航，A/B/A 可恢复 A 的内存状态，返回时按最新目录、上下文摘要和版本重新校验；原生页面因读取宿主当前上下文，离开工作区即销毁。账户全屏入口在换工作区时关闭。

容器提供 `data-aio-page-active`、`data-aio-workspace-active`、`data-aio-workspace` 和 `data-aio-workspace-context`。宿主必须在工作区离开时阻止后台服务调用并撤销票据，返回后重新鉴权，仅页面状态可保留；这些 DOM 标记不是服务端授权依据。壳不读取或管理插件内部状态，也不承诺完整刷新后保留内存。

## 可分享导航与页面状态

`PluginApplication` 将当前业务页面写入 `?page=<稳定页面ID>`，账户全屏页使用 `account` 参数；场景由页面定义派生，不维护另一套场景状态。初次解析地址后才挂载业务页面，前进后退从 URL 恢复。不存在或未授权的页面显示错误，URL 不构成权限凭证。工作区切换沿用宿主的鉴权流程，只在内存中隔离各工作区的视图缓存，不把登录上下文或挂载票据放进链接。

原生页面通过 `use_page_url_state("file-list")` 读取 `value("category")` 等业务字段，通过 `update(&[("category", Some(value))], UrlUpdate::Push)` 提交。业务参数统一使用 `view.` 命名空间，页面切换不会误删宿主或跟踪参数；同一动作的相关字段一次提交。默认值用 `None` 省略；搜索输入使用 `Continuous` 将同一输入批次合并为一个历史节点，校正非法参数使用 `Replace`。

插件必须显式声明可分享字段，禁止序列化整个 store、凭据或未保存表单。iframe 宿主通过 `window.__adminUrlState` 的 `parameter`、`update` 与 `admin-url-state` 事件接入已有安全桥，分享的是外层 URL 中的 `route`，不是包含临时票据的 iframe 资产地址。

窗口和壳内容区滚动自动恢复；内部滚动容器声明稳定的 `data-url-scroll="files-list"`。`scroll` 保存有界位置，滚动合并使用 replace，不创建历史节点；新页面和历史恢复等待容器及数据就绪。偏移只是尽力恢复，不保证数据增删或设备尺寸变化后定位到同一条记录，需要精确定位时应增加稳定记录锚点。

页面容器的 `.application-page` 继承内容区可用高度，前端插件 iframe 消费共享 `.application-frontend` 样式铺满容器，不设置固定像素高度。插件负责其内部滚动和弹窗布局；壳仍负责原生长页面滚动及桌面、移动端外边距。

不需要元数据工作台时关闭默认 feature，依赖中不会包含 Provider 注册运行时：

```toml
[dependencies]
az-dioxus-admin-shell = { version = "2026.8.17", default-features = false }
```

需要场景、菜单和页面元数据编辑能力时启用默认的 `workbench` feature，并由消费方实现、注册一个 `AdminProvider`，然后启动 `az_dioxus_admin_shell::App`。CRUD 由 `az-dioxus-admin-extension-crud` 独立提供，完整示例位于仓库 `examples/web`。
# 设置中心扩展

宿主在设置中心外提供 `ApplicationSettings` 上下文。`selected` 在宿主范围保存当前分组，避免目录刷新导致原生设置页面重建时丢失选择；`groups` 是当前已授权的 `ApplicationSettingsGroup` 响应式列表；`render.call(page_id)` 挂载选中分组。设置中心只负责选择与展示，宿主负责版本、用户和租户隔离。只挂载选中项，卸载或撤权后由列表更新移除入口。
