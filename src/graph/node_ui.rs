
use std::{ops::{RangeInclusive}, path::Path, rc::{Rc, Weak}, cell::RefCell};

use egui::{DragValue, color_picker::{color_edit_button_rgba}, Slider, color::Hsva, RichText, Color32, Stroke, Label, Sense, InnerResponse, Response, Ui, Area, Frame, Id, Order, Layout, Align, Widget, Vec2};
use egui_node_graph::{Graph, NodeDataTrait, NodeId, WidgetValueTrait, DataTypeTrait};

use serde::{Serialize, Deserialize};

use crate::{common::{def::{ConnectionType, UiValue, RangedData, TextStyle}, ui_util::{horizontal_drags, UiLimit}, animation::DataUpdater}, textures::UiTexture};

use super::def::*;

fn draw_error(ui: &mut egui::Ui, name: &str, error: &Option<NodeError>){
    if let Some(error) = &error {

        // let err_time_diff = error.when.elapsed();
        let err_elapsed_s = error.when.elapsed().as_secs_f32();
        // error.when.elapsed()

        let error_is_recent = err_elapsed_s < 1.0;

        let color = if error_is_recent {
            Color32::RED
        } else {
            Color32::GRAY
        };

        egui::Frame::none()
            .inner_margin(2.0)
            .stroke(Stroke::new(1.0, color))
            .show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                ui.label(RichText::new(format!("Error in {name}")).code().color(Color32::LIGHT_RED));
                ui.label(RichText::new(format!("{err_elapsed_s:.2}s ago")).small());
                ui.add(Label::new(RichText::new(&error.text).code()).sense(Sense::click_and_drag()));
            });
    }
}

enum ImageScale {
    MaxWidth(f32),
    MaxSize(f32)
}

fn show_image(ui: &mut Ui, texture: Weak<RefCell<UiTexture>>, scale: ImageScale) -> Response {
    egui::Frame::none()
        .stroke(Stroke::new(1.0, Color32::BLACK))
        .show(ui, |ui| {
            // ui.set_min_size(ui.available_size());

            if let Some(tex) = texture.upgrade() {
                let tex = tex.borrow();

                // let width = 200.0;
                let (tex_w, tex_h) = tex.size();
                let tex_size = glam::Vec2::new(tex_w as f32, tex_h as f32);

                let img_size = match scale {
                    ImageScale::MaxWidth(width) => {
                        let height = tex_size.x * width / tex_size.y;
                        glam::Vec2::new(width, height)
                    },
                    ImageScale::MaxSize(max_size) => {
                        glam::Vec2::new(tex_size.x, tex_size.y).clamp_length_max(max_size)
                    },
                };
                
                // Vec2::ONE.m
                // let height = tex_h as f32 * width / tex_w as f32;
    
                ui.image(tex.clone_screen_tex_id(), img_size.to_array())
            } else {
                ui.label("NO IMAGE AVAILABLE")
            }
        }).response
}

impl NodeDataTrait for NodeData {
    type Response = GraphResponse;
    type UserState = GraphState;
    type DataType = ConnectionType;
    type ValueType = UiValue;

    fn bottom_ui(
        &self,
        ui: &mut egui::Ui,
        node_id: NodeId,
        graph: &Graph<Self, Self::DataType, Self::ValueType>,
        state: &mut Self::UserState,
    ) -> Vec<egui_node_graph::NodeResponse<Self::Response, Self>>
    where
        Self::Response: egui_node_graph::UserResponseTrait,
    {
        let node = &graph[node_id];

        let show_tex = state.visible_nodes.contains(&node_id);

        if show_tex {
            if show_image(ui, node.user_data.texture.clone(), ImageScale::MaxWidth(ui.available_width())).interact(egui::Sense::click()).clicked() {
                state.visible_nodes.remove(&node_id);
            }
            ;
        } else {
            if show_image(ui, node.user_data.texture.clone(), ImageScale::MaxSize(50.0)).interact(egui::Sense::click()).clicked() {
                state.visible_nodes.insert(node_id);
            }
        }

        if ui.ui_contains_pointer() {
            egui::show_tooltip_at_pointer(ui.ctx(), egui::Id::new("img_hover"), |ui| {
                show_image(ui, node.user_data.texture.clone(), ImageScale::MaxSize(200.0))
            });
        }

        draw_error(ui, "Init", &node.user_data.create_error);
        draw_error(ui, "Update", &node.user_data.update_error);
        draw_error(ui, "Render", &node.user_data.render_error);
        
        vec![]
    }
}

impl DataTypeTrait<GraphState> for ConnectionType {
    fn data_type_color(&self, _: &mut GraphState) -> egui::Color32 {
        let hue = match self {
            ConnectionType::Texture2D => 0.7,
            ConnectionType::None => 0.0,
        };

        Hsva::new(hue, 1., 1., 1.).into()
    }

    fn name(&self) -> std::borrow::Cow<str> {
        self.to_string().into()
    }
}


// fn horizontal_drags_arr()

fn default_range_f32(min: &Option<f32>, max: &Option<f32>) -> RangeInclusive<f32>{
    min.unwrap_or(0.0)..=max.unwrap_or(1.0)
}

fn default_range_i32(min: &Option<i32>, max: &Option<i32>) -> RangeInclusive<i32>{
    min.unwrap_or(0)..=max.unwrap_or(1)
}

#[derive(Serialize, Deserialize)]
pub enum UpdaterUiState {
    None,
    Editing
}

struct ParamUiResponse {
    response: Response,
    changed: bool
}

impl From<InnerResponse<Response>> for ParamUiResponse {
    fn from(value: InnerResponse<Response>) -> Self {
        Self {
            changed: value.response.changed() || value.inner.changed(),
            response: value.response
        }
    }
}

impl From<InnerResponse<bool>> for ParamUiResponse {
    fn from(value: InnerResponse<bool>) -> Self {
        Self {
            changed: value.response.changed() || value.inner,
            response: value.response
        }
    }
}

impl From<Response> for ParamUiResponse {
    fn from(value: Response) -> Self {
        Self {
            changed: value.changed(),
            response: value
        }
    }
}

pub fn popup<R>(
    ui: &Ui,
    popup_id: Id,
    widget_response: &Response,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    Area::new(popup_id)
        .order(Order::Foreground)
        .fixed_pos(widget_response.rect.left_bottom())
        .show(ui.ctx(), |ui| {
            // Note: we use a separate clip-rect for this area, so the popup can be outside the parent.
            // See https://github.com/emilk/egui/issues/825
            let frame = Frame::popup(ui.style());
            let frame_margin = frame.inner_margin + frame.outer_margin;
            frame
                .show(ui, |ui| {
                    ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
                        ui.set_width(widget_response.rect.width() - frame_margin.sum().x);
                        add_contents(ui)
                    })
                    .inner
                })
                .inner
        })
}

impl WidgetValueTrait for UiValue {
    type Response = GraphResponse;
    type UserState = GraphState;
    type NodeData = NodeData;

    fn value_widget(
        &mut self,
        param_name: &str,
        node_id: NodeId,
        ui: &mut egui::Ui,
        user_state: &mut Self::UserState,
        _node_data: &Self::NodeData,
    ) -> Vec<Self::Response> {

        let param_response: ParamUiResponse = match self {
            UiValue::Vec2 (data) => {
                ui.label(param_name);
                horizontal_drags(
                    ui, 
                    &["x", "y"], 
                    UiLimit::Clamp(data.min.as_ref(), data.max.as_ref()),
                    // ,
                    // ,
                    &mut data.value, 
                ).into()
            }

            UiValue::Vec4(data) => {
                ui.label(param_name);
                horizontal_drags(
                    ui, 
                    &["r", "g", "b", "a"], 
                    UiLimit::Clamp(data.min.as_ref(), data.max.as_ref()),
                    &mut data.value, 
                ).into()
            }

            UiValue::Color(RangedData { value, .. }) => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    color_edit_button_rgba(ui, value, egui::color_picker::Alpha::OnlyBlend)
                }).into()
            }

            UiValue::Float (RangedData { value, min, max, .. }) => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    // ui.add(DragValue::new(value))
                    ui.add(Slider::new(value, default_range_f32(min, max)).clamp_to_range(false))
                }).into()
            }

            UiValue::Long(RangedData { value, min, max, .. }) => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.add(DragValue::new(value).clamp_range(default_range_i32(min, max)))
                }).into()
            },

            UiValue::Bool(RangedData { value, .. }) => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.checkbox(value, "")
                }).into()
            }

            UiValue::Path(path) => {
                ui.horizontal(|ui| {
                    ui.label(param_name);

                    let path_text = if let Some(path) = path {
                        if let Some(path_str) = path.to_str() {
                            let max_length = 30;

                            if max_length < path_str.len() {
                                &path_str[path_str.len()-max_length..]
                            } else {
                                path_str
                            }
                        } else {
                            "???"
                        }
                    } else {
                        "Open"
                    };
                    let open_resp = ui.button(path_text);

                    if ui.ui_contains_pointer() {
                        let files = &ui.ctx().input().raw.dropped_files;
                        if let Some(file) = files.iter().next() {
                            if file.path.is_some() {
                                *path = file.path.clone();
                            }
                        }
                    }

                    if open_resp.clicked() {
                        let open_dir = path.as_deref().map(Path::to_str).flatten().unwrap_or(&"~");

                        let new_path = native_dialog::FileDialog::new()
                            .set_location(open_dir)
                            .add_filter("OBJ file", &["obj"])
                            // .add_filter("JPEG Image", &["jpg", "jpeg"])
                            .show_open_single_file()
                            .unwrap();

                        if new_path.is_some() {
                            *path = new_path;
                        }
                    }

                    open_resp
                }).into()
            }

            UiValue::Mat4(mat) => {
                let mut changed = false;

                ui.vertical(|ui| {
                    ui.label(param_name);

                    ui.horizontal(|ui| {
                        ui.label("s");
                        changed |= ui.add(DragValue::new(&mut mat.scale)).changed()
                    });

                    // let tx
                    // let mut slice = mat.translation.to_array();

                    changed |= horizontal_drags(
                        ui, 
                        &["tx", "ty", "tz"], 
                        UiLimit::None,
                        &mut mat.translation
                    ).inner;

                    // mat.translation = Vec3::from_slice(&slice);
                    changed |= horizontal_drags(
                        ui, 
                        &["rx", "ry", "rz"], 
                        UiLimit::Loop(&[0f32; 3], &[360f32; 3]),
                        &mut mat.rotation
                    ).inner;

                    if changed {
                        mat.update_mat();
                    }

                    changed
                }).into()
            }

            UiValue::Text(RangedData { value, .. }, style) => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    let widget = match style {
                        TextStyle::Oneline => egui::TextEdit::singleline(value),
                        TextStyle::Multiline => egui::TextEdit::multiline(value).code_editor()
                    };
                    ui.set_max_width(256.0);
                    ui.add_sized(ui.available_size(), widget)
                }).into()
            }

            UiValue::None => { ui.label(param_name).into() }
        };

        let param_key = (node_id, param_name.to_string());

        let animator_popup_id = ui.make_persistent_id(param_key.clone());

        if ui.rect_contains_pointer(param_response.response.rect) && ui.input().pointer.secondary_clicked() {
            user_state.editing_param = Some(param_key.clone());
        }
        
        if user_state.editing_param.as_ref() == Some(&param_key) {
            let popup_response = popup(&ui, 
                animator_popup_id, 
                &param_response.response,
                |ui|{
                    ui.horizontal(|ui| {
    
                        let animator = user_state.animations.get_mut(&param_key);
                        let mut delete = false;
                        match animator {
                            Some(updater) => {
                                delete |= ui.button("REMOVE").clicked();
                                updater.ui(ui);
                            },
                            None => {
                                if ui.button("ANIMATE").clicked() {
                                    if let Some(updater) = DataUpdater::from_param(self) {
                                        user_state.animations.insert(param_key.clone(), updater);
                                    }
                                }
                            },
                        }
        
                        if delete {
                            user_state.animations.remove(&param_key);
                        }
                    })
                }
            );

            if popup_response.response.clicked_elsewhere() && param_response.response.clicked_elsewhere() {
                user_state.editing_param = None;
            }
        }

        if param_response.changed {
            user_state.editing_param = None;
        }

        // if let Some(popup_response) = popup_response {
        //     if ui.rect_contains_pointer(popup_response.response.rect) {
        //         ui.memory().open_popup(animator_popup_id); 
        //     }
        // }

        // if let Some(popup_response) = popup_response {
        //     let input = ui.input();
        //     if input.pointer.primary_clicked() && !ui.rect_contains_pointer(popup_response.response.rect) {
        //         user_state.editing_param = None;
        //     }
        // }


        vec![]
    }
}