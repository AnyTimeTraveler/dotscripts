use crate::swaymsg::{self, SwayOutput};
use crate::{BG_PATH, TRANS_CROPPED};
use errors_with_context::{ErrorMessage, WithContext};
use serde::Deserialize;
use std::ops::Deref;

pub struct SwayOutputs {
    outputs: Vec<Output>,
    configs: Vec<OutputConfig>,
}

impl Deref for SwayOutputs {
    type Target = Vec<Output>;

    fn deref(&self) -> &Self::Target {
        &self.outputs
    }
}

impl SwayOutputs {
    pub(crate) async fn get_outputs() -> Result<SwayOutputs, ErrorMessage> {
        let outputs = swaymsg::get_outputs().await?;
        Ok(SwayOutputs {
            configs: outputs
                .clone()
                .into_iter()
                .enumerate()
                .map(|(i, output)| {
                    let SwayOutput { name, modes, .. } = output;
                    OutputConfig {
                        enabled: true,
                        x_offset: None,
                        y_offset: None,
                        background: None,
                        stub: OutputRef(i),
                        name,
                        modes,
                    }
                })
                .collect(),
            outputs: outputs
                .into_iter()
                .enumerate()
                .map(|(i, output)| {
                    let SwayOutput { name, make, model, serial, modes } = output;
                    Output { index: OutputRef(i), name, make, model, serial, modes }
                })
                .collect(),
        })
    }

    pub(crate) async fn setup(
        &self,
        closure: impl FnOnce(&mut OutputConfigEnv),
    ) -> Result<(), ErrorMessage> {
        let mut config = OutputConfigEnv(self.configs.clone());
        closure(&mut config);
        let mut setup_string = String::new();
        for output in config.0 {
            if output.enabled {
                let name = &output.name;
                let width = output.width();
                let height = output.height();
                let refresh = output.refresh();
                let x_offset = output.x_offset.unwrap_or(0);
                let y_offset = output.y_offset.unwrap_or(0);
                let background = output.background.unwrap_or(TRANS_CROPPED);

                setup_string += &format!(
                    r#"output "{name}" {{
                mode  {width}x{height}@{refresh}Hz
                pos {x_offset} {y_offset}
                transform normal
                scale 1.0
                scale_filter nearest
                adaptive_sync off
                dpms on
                bg {BG_PATH}/{background}
            }}
            "#
                );
            } else {
                let name = &output.name;
                setup_string += &format!(r#"output "{name}" disable"#);
            }
        }

        swaymsg::apply_setup(setup_string)
            .await
            .with_err_context("Error applying new monitor configuration")
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct OutputRef(usize);

#[derive(Clone, Eq, PartialEq)]
pub struct Output {
    pub(crate) index: OutputRef,
    pub(crate) name: String,
    pub(crate) make: String,
    pub(crate) model: String,
    pub(crate) serial: String,
    pub(crate) modes: Vec<Mode>,
}

impl Deref for Output {
    type Target = OutputRef;

    fn deref(&self) -> &Self::Target {
        &self.index
    }
}

impl Output {
    pub(crate) fn width(&self) -> u32 {
        self.modes[0].width
    }

    pub(crate) fn height(&self) -> u32 {
        self.modes[0].height
    }

    #[allow(unused)]
    pub(crate) fn refresh(&self) -> f32 {
        self.modes[0].refresh as f32 / 1000.0
    }
}

#[derive(Clone, Eq, PartialEq, Deserialize)]
pub struct Mode {
    width: u32,
    height: u32,
    refresh: u32,
}

#[derive(Clone)]
pub struct OutputConfig {
    name: String,
    modes: Vec<Mode>,
    enabled: bool,
    x_offset: Option<u32>,
    y_offset: Option<u32>,
    background: Option<&'static str>,
    stub: OutputRef,
}

impl PartialEq<Self> for OutputConfig {
    fn eq(&self, other: &Self) -> bool {
        self.stub.0 == other.stub.0
    }
}

impl Eq for OutputConfig {}

impl Deref for OutputConfig {
    type Target = OutputRef;

    fn deref(&self) -> &Self::Target {
        &self.stub
    }
}

impl OutputConfig {
    pub(crate) fn width(&self) -> u32 {
        self.modes[0].width
    }

    pub(crate) fn height(&self) -> u32 {
        self.modes[0].height
    }

    pub(crate) fn refresh(&self) -> f32 {
        self.modes[0].refresh as f32 / 1000.0
    }
}

impl OutputConfig {
    pub(crate) fn x(&mut self, x: u32) -> &mut Self {
        self.x_offset = Some(x);
        self
    }

    pub(crate) fn y(&mut self, y: u32) -> &mut Self {
        self.y_offset = Some(y);
        self
    }

    pub(crate) fn bg(&mut self, background: &'static str) -> &mut Self {
        self.background = Some(background);
        self
    }

    pub(crate) fn disable(&mut self) {
        self.enabled = false;
    }
}

pub(crate) struct OutputConfigEnv(Vec<OutputConfig>);

impl OutputConfigEnv {
    pub(crate) fn config(&mut self, stub: &OutputRef) -> &mut OutputConfig {
        &mut self.0[stub.0]
    }
}
