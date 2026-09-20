# 对话工作区外观

`UiStylesheets` 加载 `style.css`、`codex.css` 与 `codex-panels.css`。业务侧只组合语义类名、状态和共享控件，不保存 CSS。

`main.dx-conversation[data-appearance="codex"]` 开启桌面对话外观：侧栏 345px（较小桌面 275px）、顶栏 46px、输入区最大宽度 752px、输入区圆角 22px；通过 `data-sidebar-collapsed` 控制桌面侧栏，`data-history` 控制移动端抽屉，`data-context` 控制右侧上下文。`.dx-conversation__main[data-empty="true"]` 将欢迎语与输入框置于起始视野。消息区和侧栏各自滚动，输入框固定在工作区内。

消费者从 `__history`、`__main`（`__header`、`__messages`、`__composer-wrap`）、可选 `__context` 三个区域开始组合。`__composer-wrap` 内可放模型快捷条 `__model-shortcuts`、路由面板 `__router`，随后放表单 `__composer` 与 `__composer-footer`；`__device-control` 位于输入区左侧，`__model-control` 位于右侧；模型、设备使用共享 `Select` 的 `SelectPlacement::Top`。`__message[data-role="user"]` 右侧气泡，助手消息使用普通 Markdown。`__history-group` 使用原生 details 分组，`__environment` 用于右侧环境信息。是否显示、展开和禁用由业务方提供真实状态，样式库不引入路由或设备行为。已有默认对话外观不受影响。

颜色支持系统深色与显式 `html[data-theme]`；正文和控件均使用系统字体。图标、品牌和业务能力由使用方提供。

尺寸、圆角和阴影以 26.915.31945（build 9922）桌面静态样式和用户提供的魔改截图为对照。2009×1184 参考视口下，输入框 x=815、y=1053、752×122，侧栏 `#fdfdfd`，消息、输入区为白色。消费者通过浏览器截图和几何测量验证桌面、手机与深色表现。这里只提供外观，不声明第三方产品功能或整张截图像素完全相同。
