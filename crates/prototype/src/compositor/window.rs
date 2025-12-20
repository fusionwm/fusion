use smithay::backend::winit::{WinitEvent, WinitEventLoop, WinitGraphicsBackend};
use smithay::output::Mode;
use smithay::{backend::renderer::gles::GlesRenderer, output};

use crate::compositor::backend::Backend;
use crate::compositor::state::App;

pub struct WinitBackend {
    pub backend: WinitGraphicsBackend<GlesRenderer>,
    pub winit: WinitEventLoop,
}

impl WinitBackend {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (backend, winit) = smithay::backend::winit::init::<GlesRenderer>()?;
        Ok(Self { backend, winit })
    }

    pub fn bind(
        &mut self,
    ) -> (
        &mut smithay::backend::renderer::gles::GlesRenderer,
        smithay::backend::renderer::gles::GlesTarget<'_>,
    ) {
        self.backend.bind().unwrap()
    }

    pub fn backend(&mut self) -> &mut WinitGraphicsBackend<GlesRenderer> {
        &mut self.backend
    }
}

impl Backend for WinitBackend {
    fn create_output(&self) -> output::Output {
        // Сообщает клиенту физические свойства выходных данных.
        // Мы полагаемся на winit для управления физическим устройством, поэтому точный размер/марка/модель не нужны.
        let physical_properties = output::PhysicalProperties {
            // Размер в милиметрах
            size: (0, 0).into(),
            // Как физические пиксели организованы (HorizontalRGB, VerticalBGR).
            // Оставляем неизвестным для обычных выходов.
            subpixel: output::Subpixel::Unknown,
            make: "mwm".into(),
            model: "Winit".into(),
        };

        // Создаем новый вывод который является областью в пространстве композитора, которую можно использовать клиентами.
        // Обычно представляет собой монитор, используемый композитором.
        output::Output::new("winit".to_string(), physical_properties)
    }

    fn mode(&self) -> output::Mode {
        // Получаем размер окна winit
        let size = self.backend.window_size();

        // Определяем размер окна и частоту обновления в милигерцах
        output::Mode {
            size,
            // 60 fps
            refresh: 60_000,
        }
    }
}
