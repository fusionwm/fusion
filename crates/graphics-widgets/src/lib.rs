#![allow(clippy::cast_precision_loss)]

pub mod button;
pub mod image;
pub mod row;
pub mod slider;
pub mod text;

#[macro_export]
macro_rules! impl_proxy_widget {
    ($name:ident, $ctx:ident) => {
        impl toolkit::widget::WidgetQuery<$ctx> for $name {
            fn get_element<QW: toolkit::widget::Widget<$ctx>>(&self, id: &str) -> Option<&QW> {
                self.0.get_element(id)
            }

            fn get_mut_element<QW: toolkit::widget::Widget<$ctx>>(
                &mut self,
                id: &str,
            ) -> Option<&mut QW> {
                self.0.get_mut_element(id)
            }

            fn id(&self) -> Option<&str> {
                self.0.id()
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self.0.as_any()
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self.0.as_any_mut()
            }
        }

        impl toolkit::widget::Widget<$ctx> for $name {
            fn anchor(&self) -> toolkit::widget::Anchor {
                self.0.anchor()
            }

            fn desired_size(&self) -> toolkit::widget::DesiredSize {
                self.0.desired_size()
            }

            fn draw<'frame>(&'frame self, out: &mut toolkit::commands::CommandBuffer<'frame>) {
                self.0.draw(out);
            }

            fn layout(&mut self, bounds: toolkit::types::Bounds) {
                self.0.layout(bounds);
            }

            fn update(
                &mut self,
                ctx: &toolkit::widget::FrameContext,
                sender: &mut toolkit::widget::Sender<$ctx>,
            ) {
                self.0.update(ctx, sender);
            }
        }
    };
}
