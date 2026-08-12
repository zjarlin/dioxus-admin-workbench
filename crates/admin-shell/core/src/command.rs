use anyhow::{Result, bail, ensure};
use serde::{Deserialize, Serialize};

use crate::{
    AdminDefinition, DefinitionId, MenuDefinition, PageDefinition, PageRendererDefinition,
    SceneDefinition,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum AdminCommand {
    SetApplicationTitle {
        title: String,
    },
    AddScene {
        scene: SceneDefinition,
    },
    AddMenu {
        scene_id: DefinitionId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        parent_menu_id: Option<DefinitionId>,
        menu: MenuDefinition,
    },
    AddPage {
        page: PageDefinition,
    },
    AddMenuPage {
        scene_id: DefinitionId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        parent_menu_id: Option<DefinitionId>,
        menu: MenuDefinition,
        page: PageDefinition,
    },
    SetPageRenderer {
        page_id: DefinitionId,
        renderer: PageRendererDefinition,
    },
    DeleteScene {
        scene_id: DefinitionId,
    },
    DeleteMenu {
        scene_id: DefinitionId,
        menu_id: DefinitionId,
    },
    DeletePage {
        page_id: DefinitionId,
    },
}

impl AdminDefinition {
    pub fn apply(&mut self, command: AdminCommand) -> Result<()> {
        match command {
            AdminCommand::SetApplicationTitle { title } => self.set_application_title(title),
            AdminCommand::AddScene { scene } => self.add_scene(scene),
            AdminCommand::AddMenu {
                scene_id,
                parent_menu_id,
                menu,
            } => self.add_menu(&scene_id, parent_menu_id.as_ref(), menu),
            AdminCommand::AddPage { page } => self.add_page(page),
            AdminCommand::AddMenuPage {
                scene_id,
                parent_menu_id,
                menu,
                page,
            } => {
                ensure!(
                    menu.page_id.as_ref() == Some(&page.id),
                    "菜单必须引用同一命令创建的页面"
                );
                ensure!(
                    self.pages.iter().all(|existing| existing.id != page.id),
                    "页面 ID 已存在: {}",
                    page.id
                );
                ensure!(
                    self.pages.iter().all(|existing| existing.name != page.name),
                    "页面标识已存在: {}",
                    page.name
                );
                self.add_menu(&scene_id, parent_menu_id.as_ref(), menu)?;
                self.pages.push(page);
                Ok(())
            }
            AdminCommand::SetPageRenderer { page_id, renderer } => {
                self.set_page_renderer(&page_id, renderer)
            }
            AdminCommand::DeleteScene { scene_id } => self.delete_scene(&scene_id),
            AdminCommand::DeleteMenu { scene_id, menu_id } => self.delete_menu(&scene_id, &menu_id),
            AdminCommand::DeletePage { page_id } => self.delete_page(&page_id),
        }
    }

    fn set_application_title(&mut self, title: String) -> Result<()> {
        let title = title.trim();
        ensure!(!title.is_empty(), "应用标题不能为空");
        self.title = title.to_owned();
        Ok(())
    }

    fn add_scene(&mut self, scene: SceneDefinition) -> Result<()> {
        ensure!(
            self.scenes.iter().all(|existing| existing.id != scene.id),
            "场景 ID 已存在: {}",
            scene.id
        );
        ensure!(
            self.scenes
                .iter()
                .all(|existing| existing.name != scene.name),
            "场景标识已存在: {}",
            scene.name
        );
        self.scenes.push(scene);
        Ok(())
    }

    fn add_menu(
        &mut self,
        scene_id: &DefinitionId,
        parent_menu_id: Option<&DefinitionId>,
        menu: MenuDefinition,
    ) -> Result<()> {
        let scene = self
            .scenes
            .iter_mut()
            .find(|scene| &scene.id == scene_id)
            .ok_or_else(|| anyhow::anyhow!("场景不存在: {scene_id}"))?;
        ensure!(
            !menu_exists(&scene.menus, &menu.id),
            "菜单 ID 已存在: {}",
            menu.id
        );
        ensure!(
            !menu_name_exists(&scene.menus, &menu.name),
            "菜单标识已存在: {}",
            menu.name
        );
        if let Some(parent_menu_id) = parent_menu_id {
            let Some(parent) = find_menu_mut(&mut scene.menus, parent_menu_id) else {
                bail!("父菜单不存在: {parent_menu_id}");
            };
            parent.children.push(menu);
            return Ok(());
        }
        scene.menus.push(menu);
        Ok(())
    }

    fn add_page(&mut self, page: PageDefinition) -> Result<()> {
        ensure!(
            self.pages.iter().all(|existing| existing.id != page.id),
            "页面 ID 已存在: {}",
            page.id
        );
        ensure!(
            self.pages.iter().all(|existing| existing.name != page.name),
            "页面标识已存在: {}",
            page.name
        );
        self.pages.push(page);
        Ok(())
    }

    fn set_page_renderer(
        &mut self,
        page_id: &DefinitionId,
        renderer: PageRendererDefinition,
    ) -> Result<()> {
        let page = self
            .pages
            .iter_mut()
            .find(|page| &page.id == page_id)
            .ok_or_else(|| anyhow::anyhow!("页面不存在: {page_id}"))?;
        page.renderer = renderer;
        Ok(())
    }

    fn delete_scene(&mut self, scene_id: &DefinitionId) -> Result<()> {
        let previous = self.scenes.len();
        self.scenes.retain(|scene| &scene.id != scene_id);
        ensure!(previous != self.scenes.len(), "场景不存在: {scene_id}");
        Ok(())
    }

    fn delete_menu(&mut self, scene_id: &DefinitionId, menu_id: &DefinitionId) -> Result<()> {
        let scene = self
            .scenes
            .iter_mut()
            .find(|scene| &scene.id == scene_id)
            .ok_or_else(|| anyhow::anyhow!("场景不存在: {scene_id}"))?;
        ensure!(
            delete_menu(&mut scene.menus, menu_id),
            "菜单不存在: {menu_id}"
        );
        Ok(())
    }

    fn delete_page(&mut self, page_id: &DefinitionId) -> Result<()> {
        let previous = self.pages.len();
        self.pages.retain(|page| &page.id != page_id);
        ensure!(previous != self.pages.len(), "页面不存在: {page_id}");
        for scene in &mut self.scenes {
            clear_page_references(&mut scene.menus, page_id);
        }
        Ok(())
    }
}

fn menu_exists(menus: &[MenuDefinition], id: &DefinitionId) -> bool {
    menus
        .iter()
        .any(|menu| &menu.id == id || menu_exists(&menu.children, id))
}

fn menu_name_exists(menus: &[MenuDefinition], name: &str) -> bool {
    menus
        .iter()
        .any(|menu| menu.name == name || menu_name_exists(&menu.children, name))
}

fn find_menu_mut<'a>(
    menus: &'a mut [MenuDefinition],
    id: &DefinitionId,
) -> Option<&'a mut MenuDefinition> {
    for menu in menus {
        if &menu.id == id {
            return Some(menu);
        }
        if let Some(found) = find_menu_mut(&mut menu.children, id) {
            return Some(found);
        }
    }
    None
}

fn delete_menu(menus: &mut Vec<MenuDefinition>, id: &DefinitionId) -> bool {
    let previous = menus.len();
    menus.retain(|menu| &menu.id != id);
    if previous != menus.len() {
        return true;
    }
    menus
        .iter_mut()
        .any(|menu| delete_menu(&mut menu.children, id))
}

fn clear_page_references(menus: &mut [MenuDefinition], page_id: &DefinitionId) {
    for menu in menus {
        if menu.page_id.as_ref() == Some(page_id) {
            menu.page_id = None;
        }
        clear_page_references(&mut menu.children, page_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn definition() -> AdminDefinition {
        AdminDefinition {
            id: DefinitionId::new(),
            name: "demo".to_owned(),
            title: "Demo".to_owned(),
            scenes: Vec::new(),
            pages: Vec::new(),
        }
    }

    #[test]
    fn deleting_page_clears_menu_reference() -> Result<()> {
        let page_id = DefinitionId::new();
        let scene_id = DefinitionId::new();
        let mut definition = definition();
        definition.apply(AdminCommand::AddPage {
            page: PageDefinition {
                id: page_id.clone(),
                name: "users".to_owned(),
                title: "用户".to_owned(),
                renderer: PageRendererDefinition::ConventionFile,
            },
        })?;
        definition.apply(AdminCommand::AddScene {
            scene: SceneDefinition {
                id: scene_id,
                name: "system".to_owned(),
                title: "系统".to_owned(),
                menus: vec![MenuDefinition {
                    id: DefinitionId::new(),
                    name: "users".to_owned(),
                    title: "用户".to_owned(),
                    icon: None,
                    page_id: Some(page_id.clone()),
                    enabled: true,
                    children: Vec::new(),
                }],
            },
        })?;

        definition.apply(AdminCommand::DeletePage { page_id })?;
        assert!(definition.scenes[0].menus[0].page_id.is_none());
        Ok(())
    }

    #[test]
    fn application_title_must_not_be_empty() -> Result<()> {
        let mut definition = definition();

        definition.apply(AdminCommand::SetApplicationTitle {
            title: "新的应用名称".to_owned(),
        })?;
        assert_eq!(definition.title, "新的应用名称");

        let result = definition.apply(AdminCommand::SetApplicationTitle {
            title: "  ".to_owned(),
        });
        assert!(result.is_err());
        assert_eq!(definition.title, "新的应用名称");
        Ok(())
    }

    #[test]
    fn generated_names_must_be_unique_before_mutation() -> Result<()> {
        let scene_id = DefinitionId::new();
        let mut definition = definition();
        definition.apply(AdminCommand::AddScene {
            scene: SceneDefinition {
                id: scene_id.clone(),
                name: "zi_chan".to_owned(),
                title: "资产".to_owned(),
                menus: Vec::new(),
            },
        })?;
        let duplicate_scene = definition.apply(AdminCommand::AddScene {
            scene: SceneDefinition {
                id: DefinitionId::new(),
                name: "zi_chan".to_owned(),
                title: "资产".to_owned(),
                menus: Vec::new(),
            },
        });
        assert!(duplicate_scene.is_err());

        let page_id = DefinitionId::new();
        definition.apply(AdminCommand::AddMenuPage {
            scene_id: scene_id.clone(),
            parent_menu_id: None,
            menu: MenuDefinition {
                id: DefinitionId::new(),
                name: "she_bei".to_owned(),
                title: "设备".to_owned(),
                icon: None,
                page_id: Some(page_id.clone()),
                enabled: true,
                children: Vec::new(),
            },
            page: PageDefinition {
                id: page_id,
                name: "she_bei".to_owned(),
                title: "设备".to_owned(),
                renderer: PageRendererDefinition::ConventionFile,
            },
        })?;
        let duplicate_page_id = DefinitionId::new();
        let duplicate_page = definition.apply(AdminCommand::AddMenuPage {
            scene_id,
            parent_menu_id: None,
            menu: MenuDefinition {
                id: DefinitionId::new(),
                name: "ling_yi_ge_cai_dan".to_owned(),
                title: "另一个菜单".to_owned(),
                icon: None,
                page_id: Some(duplicate_page_id.clone()),
                enabled: true,
                children: Vec::new(),
            },
            page: PageDefinition {
                id: duplicate_page_id,
                name: "she_bei".to_owned(),
                title: "设备".to_owned(),
                renderer: PageRendererDefinition::ConventionFile,
            },
        });
        assert!(duplicate_page.is_err());
        assert_eq!(definition.scenes[0].menus.len(), 1);
        Ok(())
    }
}
