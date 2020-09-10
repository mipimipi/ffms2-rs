use crate::index::*;
use crate::resample::*;
use crate::*;

use ffms2_sys::*;

use std::ffi::c_void;
use std::ffi::CString;
use std::path::PathBuf;

simple_enum!(
    AudioChannel,
    (
        CH_FRONT_LEFT,
        CH_FRONT_RIGHT,
        CH_FRONT_CENTER,
        CH_LOW_FREQUENCY,
        CH_BACK_LEFT,
        CH_BACK_RIGHT,
        CH_FRONT_LEFT_OF_CENTER,
        CH_FRONT_RIGHT_OF_CENTER,
        CH_BACK_CENTER,
        CH_SIDE_LEFT,
        CH_SIDE_RIGHT,
        CH_TOP_CENTER,
        CH_TOP_FRONT_LEFT,
        CH_TOP_FRONT_CENTER,
        CH_TOP_FRONT_RIGHT,
        CH_TOP_BACK_LEFT,
        CH_TOP_BACK_CENTER,
        CH_TOP_BACK_RIGHT,
        CH_STEREO_LEFT,
        CH_STEREO_RIGHT,
    )
);

simple_enum!(
    AudioDelay,
    (DELAY_NO_SHIFT, DELAY_TIME_ZERO, DELAY_FIRST_VIDEO_TRACK)
);

create_enum!(
    MatrixEncoding,
    FFMS_MatrixEncoding,
    matrix_encoding,
    (
        MATRIX_ENCODING_NONE,
        MATRIX_ENCODING_DOBLY,
        MATRIX_ENCODING_PRO_LOGIC_II,
        MATRIX_ENCODING_PRO_LOGIC_IIX,
        MATRIX_ENCODING_PRO_LOGIC_IIZ,
        MATRIX_ENCODING_DOLBY_EX,
        MATRIX_ENCODING_DOLBY_HEADPHONE,
    )
);

set_struct!(AudioProperties, audio_properties, FFMS_AudioProperties);

default_struct!(
    AudioProperties,
    audio_properties,
    FFMS_AudioProperties,
    (
        SampleFormat,
        SampleRate,
        BitsPerSample,
        Channels,
        ChannelLayout,
        NumSamples,
        FirstTime,
        LastTime,
        LastEndTime,
    ),
    (0, 0, 0, 0, 0, 0, 0.0, 0.0, 0.0)
);

set_params!(
    AudioProperties,
    audio_properties,
    (
        SampleFormat,
        SampleRate,
        BitsPerSample,
        Channels,
        ChannelLayout,
        NumSamples,
        FirstTime,
        LastTime,
        LastEndTime,
    ),
    (usize, usize, usize, usize, usize, usize, f64, f64, f64),
    (
        SampleFormat as i32,
        SampleRate as i32,
        BitsPerSample as i32,
        Channels as i32,
        ChannelLayout as i64,
        NumSamples as i64,
        FirstTime as f64,
        LastTime as f64,
        LastEndTime as f64,
    )
);

pub struct AudioSource {
    audio_source: *mut FFMS_AudioSource,
}

impl AudioSource {
    pub fn new(
        SourceFile: &PathBuf,
        Track: usize,
        Index: &Index,
        DelayMode: usize,
    ) -> Result<Self, Error> {
        let source = CString::new(SourceFile.to_str().unwrap()).unwrap();
        let mut error: Error = Default::default();
        let audio_source = unsafe {
            FFMS_CreateAudioSource(
                source.as_ptr(),
                Track as i32,
                Index.as_mut_ptr(),
                DelayMode as i32,
                error.as_mut_ptr(),
            )
        };

        if audio_source.is_null() {
            Err(error)
        } else {
            Ok(AudioSource { audio_source })
        }
    }

    pub fn GetAudioProperties(&self) -> AudioProperties {
        let audio_prop = unsafe { FFMS_GetAudioProperties(self.audio_source) };
        let ref_audio = unsafe { &*audio_prop };

        AudioProperties {
            audio_properties: *ref_audio,
        }
    }

    pub fn GetAudio<T>(
        &self,
        Start: usize,
        Count: usize,
    ) -> Result<Vec<T>, Error> {
        let mut Buf: Vec<T> = Vec::new();
        let mut error: Error = Default::default();
        let audio_prop = unsafe { FFMS_GetAudioProperties(self.audio_source) };
        let num_sample = unsafe { (*audio_prop).NumSamples };

        if Start as i64 > (num_sample - 1) || Count as i64 > (num_sample - 1) {
            panic!("Requesting samples beyond the stream end");
        }

        let err = unsafe {
            FFMS_GetAudio(
                self.audio_source,
                Buf.as_mut_ptr() as *mut c_void,
                Start as i64,
                Count as i64,
                error.as_mut_ptr(),
            )
        };

        if err != 0 {
            Err(error)
        } else {
            Ok(Buf)
        }
    }

    pub fn CreateResampleOptions(&self) -> ResampleOptions {
        let res_opt = unsafe { FFMS_CreateResampleOptions(self.audio_source) };
        let ref_res = unsafe { &*res_opt };

        ResampleOptions::create_struct(ref_res)
    }

    pub fn SetOutputFormatA(
        &self,
        options: &ResampleOptions,
    ) -> Result<(), Error> {
        let mut error: Error = Default::default();
        let err = unsafe {
            FFMS_SetOutputFormatA(
                self.audio_source,
                options.as_ptr(),
                error.as_mut_ptr(),
            )
        };

        if err != 0 {
            Err(error)
        } else {
            Ok(())
        }
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut FFMS_AudioSource {
        self.audio_source
    }
}

impl Drop for AudioSource {
    fn drop(&mut self) {
        unsafe {
            FFMS_DestroyAudioSource(self.audio_source);
        }
    }
}
