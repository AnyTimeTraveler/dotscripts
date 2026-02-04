use crate::outputs::{Output, SwayOutputs};

pub(crate) struct OutputFilter {
    name_regex: Option<&'static str>,
    model_regex: Option<&'static str>,
    make_regex: Option<&'static str>,
    serial_regex: Option<&'static str>,
}

impl OutputFilter {
    pub(crate) fn new() -> Self {
        Self { name_regex: None, model_regex: None, make_regex: None, serial_regex: None }
    }
    pub(crate) fn name_regex(mut self, name_regex: &'static str) -> Self {
        self.name_regex = Some(name_regex);
        self
    }
    pub(crate) fn model_regex(mut self, model_regex: &'static str) -> Self {
        self.model_regex = Some(model_regex);
        self
    }
    pub(crate) fn make_regex(mut self, make_regex: &'static str) -> Self {
        self.make_regex = Some(make_regex);
        self
    }
    #[allow(unused)]
    pub(crate) fn serial_regex(mut self, serial_regex: &'static str) -> Self {
        self.serial_regex = Some(serial_regex);
        self
    }
}

pub(crate) fn any_of<T>(outputs: Vec<Option<T>>) -> Option<T> {
    outputs.into_iter().filter_map(|a| a).next()
}

impl SwayOutputs {
    pub(crate) fn find_monitor(&self, monitor_config: OutputFilter) -> Option<&Output> {
        for output in self.iter() {
            if let Some(model_regex) = monitor_config.model_regex {
                if output.model.matches(model_regex).next().is_none() {
                    continue;
                }
            }
            if let Some(make_regex) = monitor_config.make_regex {
                if output.make.matches(make_regex).next().is_none() {
                    continue;
                }
            }
            if let Some(name_regex) = monitor_config.name_regex {
                if output.name.matches(name_regex).next().is_none() {
                    continue;
                }
            }
            if let Some(serial_regex) = monitor_config.serial_regex {
                if output.serial.matches(serial_regex).next().is_none() {
                    continue;
                }
            }
            return Some(output);
        }
        None
    }
}
