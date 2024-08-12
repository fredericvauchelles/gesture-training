use jni::{JNIEnv, JavaVM};
use jni::objects::{JObject, JValue};

pub struct Android {

}

impl Android {
    pub fn with_current_env(f: impl FnOnce(&JNIEnv)) {
        let cx = ndk_context::android_context();
        let vm = unsafe { JavaVM::from_raw(cx.vm().cast()) }.unwrap();
        let env = vm.attach_current_thread().unwrap();

        f(&*env);
    }


    pub fn intent_open_file() -> anyhow::Result<()> {
        let cx = ndk_context::android_context();
        let vm = unsafe { JavaVM::from_raw(cx.vm().cast()) }?;
        let mut env = vm.attach_current_thread()?;
        let activity = unsafe { JObject::from_raw(cx.context() as jni::sys::jobject) };

        let intent = {
            let intent_class = env.find_class("android/content/Intent")?;
            let action_view =
                env.get_static_field(&intent_class, "ACTION_GET_CONTENT", "Ljava/lang/String;")?;

            env.new_object(intent_class, "(Ljava/lang/String;)V", &[(&action_view).into()])?
        };

        let bundle = {
            let bundle_class = env.find_class("android/os/Bundle")?;
            env.new_object(bundle_class, "()V", &[])?
        };

        env.call_method(
            activity,
            "startActivityForResult",
            "(Landroid/content/Intent;I;Landroid/os/Bundle;)V",
            &[(&intent).into(), JValue::from(1), (&bundle).into()]
        )?;

        Ok(())
    }
}