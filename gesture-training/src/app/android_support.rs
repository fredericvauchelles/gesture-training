use jni::JavaVM;
use jni::objects::{JObject, JValue};
use slint::android::AndroidApp;

pub trait AndroidSupport {
    fn request_file_permissions(&self) -> anyhow::Result<()>;
    fn intent_open_file() -> anyhow::Result<()>;

    fn vm(&self) -> anyhow::Result<JavaVM>;
    fn activity(&self) -> JObject;
}

impl AndroidSupport for AndroidApp {
    fn request_file_permissions(&self) -> anyhow::Result<()> {
        let vm = self.vm()?;
        let mut env = vm.attach_current_thread()?;
        let activity = self.activity();

        let permission_class = env.find_class("android/Manifest$permission")?;

        let read_permission = env.get_static_field(&permission_class, "READ_EXTERNAL_STORAGE", "Ljava/lang/String;")?;

        {
            let string_class = env.find_class("java/lang/String")?;
            let default_string = env.new_string("")?;
            let permissions_array = env.new_object_array(1, string_class, default_string)?;
            env.set_object_array_element(&permissions_array, 0, read_permission.l()?)?;

            env.call_method(
                activity,
                "requestPermissions",
                "([Ljava/lang/String;I)V",
                &[(&permissions_array).into(), JValue::from(0)]
            )?;
        }
        Ok(())
    }

    fn intent_open_file() -> anyhow::Result<()> {
        let cx = ndk_context::android_context();
        let vm = unsafe { JavaVM::from_raw(cx.vm().cast()) }?;
        let mut env = vm.attach_current_thread()?;
        let activity = unsafe { JObject::from_raw(cx.context() as jni::sys::jobject) };

        let intent = {
            let intent_class = env.find_class("android/content/Intent")?;
            let action_view =
                env.get_static_field(&intent_class, "ACTION_GET_CONTENT", "Ljava/lang/String;")?;

            let intent = env.new_object(intent_class, "(Ljava/lang/String;)V", &[(&action_view).into()])?;

            let set_type_arg = env.new_string("folder/*")?;

            env.call_method(
                &intent,
                "setType",
                "(Ljava/lang/String;)Landroid/content/Intent;",
                &[(&set_type_arg).into()]
            )?;

            intent
        };

        env.call_method(
            activity,
            "startActivity",
            "(Landroid/content/Intent;)V",
            &[(&intent).into()]
        )?;

        Ok(())
    }

    fn vm(&self) -> anyhow::Result<JavaVM> {
        {
            unsafe {
                JavaVM::from_raw(self.vm_as_ptr().cast())
            }.map_err(anyhow::Error::from)
        }
    }

    fn activity(&self) -> JObject {
        unsafe { JObject::from_raw(self.activity_as_ptr() as jni::sys::jobject) }
    }
}
