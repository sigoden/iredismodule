use std::convert::TryInto;
use std::os::raw::c_void;
use std::time::Duration;

use crate::raw;
use crate::{handle_status, take_data, Context, Error, TimerID};

impl Context {
    pub fn create_timer<F, T>(&self, period: Duration, callback: F, data: T) -> TimerID
    where
        F: FnOnce(&Context, T),
    {
        // Store the user-provided data on the heap before passing ownership of it to Redis,
        // so that it will outlive the current scope.
        let data = Box::into_raw(Box::from(TimerProcData { data, callback }));

        let timer_id = unsafe {
            raw::RedisModule_CreateTimer.unwrap()(
                self.inner,
                period
                    .as_millis()
                    .try_into()
                    .expect("Value must fit in 64 bits"),
                Some(timer_proc::<F, T>),
                data as *mut c_void,
            )
        };

        timer_id as TimerID
    }
    pub fn stop_timer<T>(&self, id: TimerID) -> Result<T, Error> {
        let mut data: *mut c_void = std::ptr::null_mut();

        handle_status(
            unsafe { raw::RedisModule_StopTimer.unwrap()(self.inner, id, &mut data) },
            "Cloud not stop timer",
        )?;

        let data: T = take_data(data);
        return Ok(data);
    }

    pub fn get_timer_info<T>(&self, id: TimerID) -> Result<(Duration, &T), Error> {
        let mut remaining: u64 = 0;
        let mut data: *mut c_void = std::ptr::null_mut();

        handle_status(
            unsafe {
                raw::RedisModule_GetTimerInfo.unwrap()(self.inner, id, &mut remaining, &mut data)
            },
            "Cloud not get timer info",
        )?;

        // Cast the *mut c_void supplied by the Redis API to a raw pointer of our custom type.
        let data = data as *mut T;

        // Dereference the raw pointer (we know this is safe, since Redis should return our
        // original pointer which we know to be good) and turn it into a safe reference
        let data = unsafe { &*data };

        Ok((Duration::from_millis(remaining), data))
    }
}

extern "C" fn timer_proc<F, T>(ctx: *mut raw::RedisModuleCtx, data: *mut c_void)
where
    F: FnOnce(&Context, T),
{
    let ctx = &Context::from_ptr(ctx);
    if data.is_null() {
        ctx.log_debug("Timer callback data is null");
        return;
    }
    let cb_data: TimerProcData<F, T> = take_data(data);
    (cb_data.callback)(ctx, cb_data.data);
}

#[repr(C)]
pub(crate) struct TimerProcData<F: FnOnce(&Context, T), T> {
    data: T,
    callback: F,
}
