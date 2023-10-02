use super::{graph_utils::InputParams, node_shader::NodeShader, node_types::NodeType};
use crate::common::def::UiValue;
use crate::{
    gl_expression::GlExpressionUpdater, isf::updater::IsfUpdater, obj_shader::loader::ObjLoader,
};
use glium::backend::Facade;
use std::time::SystemTime;

pub enum NodeUpdate {
    Isf(IsfUpdater),
    Obj(ObjLoader),
    Expression(GlExpressionUpdater),
}

//TODO: Only run on change (ui etc)
//Maybe time to use ECS?

impl NodeUpdate {
    pub fn new(template: &NodeType) -> Option<Self> {
        match template {
            NodeType::Isf { .. } => Some(Self::Isf(IsfUpdater {
                modified: SystemTime::now(),
            })),
            NodeType::ObjRender => Some(Self::Obj(ObjLoader::new())),
            NodeType::Expression { .. } => {
                Some(Self::Expression(GlExpressionUpdater { frag_source: None }))
            }
            _ => None,
        }
    }

    pub fn update(
        &mut self,
        facade: &impl Facade,
        template: &mut NodeType,
        inputs: &InputParams<'_>,
        shader: &mut NodeShader,
    ) -> anyhow::Result<()> {
        match (self, template, shader) {
            (
                NodeUpdate::Isf(updater),
                NodeType::Isf { info: isf_info },
                NodeShader::Isf(shader),
            ) => {
                updater.reload_if_updated(facade, isf_info, shader)?;
            }

            (NodeUpdate::Obj(loader), _, NodeShader::Obj(obj_renderer)) => {
                if let Some(Some(path)) =
                    inputs.iter().find_map(|(_name, input)| match &input.value {
                        UiValue::Path(path) => Some(path),
                        _ => None,
                    })
                {
                    loader.load_if_changed(facade, &path, obj_renderer)?;
                }
            }

            (
                NodeUpdate::Expression(updater),
                NodeType::Expression { .. },
                NodeShader::Expression(renderer),
            ) => {
                if let Some(frag_source) = inputs.iter().find_map(|(_name, val)| {
                    if let UiValue::Text(text, ..) = &val.value {
                        Some(text.value.clone())
                    } else {
                        None
                    }
                }) {
                    let _inputs = updater.update(facade, renderer, frag_source)?;
                    // dbg!(inputs);
                }
            }
            _ => {}
        }

        Ok(())
    }
}
