pub struct Lockdown {
    disable_printing: bool,
    disable_save_to_disk: bool,
    disable_application_handlers: bool,
    disable_location: bool,
    disable_camera: bool,
    disable_microphone: bool,
    disable_sound_output: bool,
}

impl Default for Lockdown {
    fn default() -> Self {
        Self {
            disable_printing: false,
            disable_save_to_disk: false,
            disable_application_handlers: false,
            disable_location: true,
            disable_camera: false,
            disable_microphone: false,
            disable_sound_output: false,
        }
    }
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Lockdown")]
impl Lockdown {
    #[zbus(property, name = "disable-printing")]
    fn disable_printing(&self) -> bool {
        self.disable_printing
    }
    #[zbus(property, name = "disable-printing")]
    fn set_disable_printing(&mut self, v: bool) {
        self.disable_printing = v;
    }

    #[zbus(property, name = "disable-save-to-disk")]
    fn disable_save_to_disk(&self) -> bool {
        self.disable_save_to_disk
    }
    #[zbus(property, name = "disable-save-to-disk")]
    fn set_disable_save_to_disk(&mut self, v: bool) {
        self.disable_save_to_disk = v;
    }

    #[zbus(property, name = "disable-application-handlers")]
    fn disable_application_handlers(&self) -> bool {
        self.disable_application_handlers
    }
    #[zbus(property, name = "disable-application-handlers")]
    fn set_disable_application_handlers(&mut self, v: bool) {
        self.disable_application_handlers = v;
    }

    #[zbus(property, name = "disable-location")]
    fn disable_location(&self) -> bool {
        self.disable_location
    }
    #[zbus(property, name = "disable-location")]
    fn set_disable_location(&mut self, v: bool) {
        self.disable_location = v;
    }

    #[zbus(property, name = "disable-camera")]
    fn disable_camera(&self) -> bool {
        self.disable_camera
    }
    #[zbus(property, name = "disable-camera")]
    fn set_disable_camera(&mut self, v: bool) {
        self.disable_camera = v;
    }

    #[zbus(property, name = "disable-microphone")]
    fn disable_microphone(&self) -> bool {
        self.disable_microphone
    }
    #[zbus(property, name = "disable-microphone")]
    fn set_disable_microphone(&mut self, v: bool) {
        self.disable_microphone = v;
    }

    #[zbus(property, name = "disable-sound-output")]
    fn disable_sound_output(&self) -> bool {
        self.disable_sound_output
    }
    #[zbus(property, name = "disable-sound-output")]
    fn set_disable_sound_output(&mut self, v: bool) {
        self.disable_sound_output = v;
    }
}
