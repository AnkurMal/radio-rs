#![allow(unused, non_snake_case, non_camel_case_types)]
use std::ffi::{CString, OsStr};
use std::marker::PhantomData;
use std::os::raw::*;
use std::time::{Duration, Instant};
use std::{path::Path, thread};

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Wave {
    pub frame_count: c_uint,
    pub sample_rate: c_uint,
    pub sample_size: c_uint,
    pub channels: c_uint,
    data: *mut c_void,
}

#[derive(Debug)]
pub struct AudioDevice {
    last_frame_time: Instant,
}

// for opaque structs - https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs (credit)
#[repr(C)]
pub struct rAudioBuffer {
    _data: [u8; 0],
    _marker: PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[repr(C)]
struct rAudioProcessor {
    _data: [u8; 0],
    _marker: PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct AudioStream {
    buffer: *mut rAudioBuffer,
    processor: *mut rAudioProcessor,
    pub sample_rate: c_uint,
    pub sample_size: c_uint,
    pub channels: c_uint,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Sound {
    pub stream: AudioStream,
    pub frame_count: c_uint,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Music {
    pub stream: AudioStream,
    pub frame_count: c_uint,
    looping: bool,
    ctx_type: c_int,
    ctx_data: *mut c_void,
}

impl Wave {
    pub fn load(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        if !path.exists() {
            panic!("File doesn't exist.");
        }

        match path.extension().and_then(OsStr::to_str).unwrap() {
            "wav" => {
                let file = CString::new(path.to_str().unwrap()).unwrap();
                unsafe { LoadWave(file.as_ptr()) }
            }
            _ => panic!("Unsupported file format"),
        }
    }

    pub fn is_ready(&self) -> bool {
        unsafe { IsWaveReady(self.clone()) }
    }

    pub fn export(&self, file_name: impl AsRef<Path>) {
        let file = CString::new(file_name.as_ref().to_str().unwrap()).unwrap();
        unsafe {
            ExportWave(self.clone(), file.as_ptr());
        }
    }

    pub fn crop(&mut self, init_sample: i32, final_sample: i32) {
        unsafe {
            WaveCrop(
                self as *mut Wave,
                init_sample as c_int,
                final_sample as c_int,
            );
        }
    }

    pub fn format(&mut self, sample_rate: i32, sample_size: i32, channels: i32) {
        unsafe {
            WaveFormat(
                self as *mut Wave,
                sample_rate as c_int,
                sample_size as c_int,
                channels as c_int,
            );
        }
    }
}

impl Drop for Wave {
    fn drop(&mut self) {
        unsafe {
            UnloadWave(self.clone());
        }
    }
}

impl Default for AudioDevice {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioDevice {
    const FPS: u64 = 60;
    const FRAME_DURATION: Duration = Duration::from_millis(1000 / Self::FPS);

    pub fn new() -> Self {
        unsafe {
            InitAudioDevice();
        }
        AudioDevice {
            last_frame_time: Instant::now(),
        }
    }

    pub fn is_ready(&self) -> bool {
        unsafe { IsAudioDeviceReady() }
    }

    pub fn sync(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_frame_time);

        if elapsed < Self::FRAME_DURATION {
            thread::sleep(Self::FRAME_DURATION - elapsed);
        }

        self.last_frame_time = Instant::now();
    }

    pub fn set_master_volume(&self, volume: f32) {
        unsafe {
            SetMasterVolume(volume as c_float);
        }
    }

    pub fn get_master_volume(&self) -> f32 {
        let volume = unsafe { GetMasterVolume() };
        volume as f32
    }
}

impl Drop for AudioDevice {
    fn drop(&mut self) {
        unsafe {
            CloseAudioDevice();
        }
    }
}

impl Sound {
    pub fn load(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        if !path.exists() {
            panic!("File doesn't exist.");
        }

        match path.extension().and_then(OsStr::to_str).unwrap() {
            "wav" | "qoa" | "ogg" | "mp3" | "flac" => {
                let file = CString::new(path.to_str().unwrap()).unwrap();
                unsafe { LoadSound(file.as_ptr()) }
            }
            _ => panic!("Unsupported file format"),
        }
    }

    pub fn load_from_wave(wave: &Wave) -> Self {
        unsafe { LoadSoundFromWave(wave.clone()) }
    }

    pub fn is_ready(&self) -> bool {
        unsafe { IsSoundReady(self.clone()) }
    }

    pub fn play(&self) {
        unsafe {
            PlaySound(self.clone());
        }
    }

    pub fn stop(&self) {
        unsafe {
            StopSound(self.clone());
        }
    }

    pub fn pause(&self) {
        unsafe {
            PauseSound(self.clone());
        }
    }

    pub fn resume(&self) {
        unsafe {
            ResumeSound(self.clone());
        }
    }

    pub fn is_playing(&self) -> bool {
        unsafe { IsSoundPlaying(self.clone()) }
    }

    pub fn set_volume(&self, volume: f32) {
        unsafe {
            SetSoundVolume(self.clone(), volume as c_float);
        }
    }

    pub fn set_pitch(&self, pitch: f32) {
        unsafe {
            SetSoundPitch(self.clone(), pitch as c_float);
        }
    }

    pub fn set_pan(&self, pan: f32) {
        unsafe {
            SetSoundPan(self.clone(), pan as c_float);
        }
    }
}

impl Drop for Sound {
    fn drop(&mut self) {
        unsafe {
            UnloadSound(self.clone());
        }
    }
}

impl Music {
    pub fn load(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        if !path.exists() {
            panic!("File doesn't exist.");
        }

        match path.extension().and_then(OsStr::to_str).unwrap() {
            "wav" | "qoa" | "ogg" | "mp3" | "flac" | "xm" => {
                let file = CString::new(path.to_str().unwrap()).unwrap();
                unsafe { LoadMusicStream(file.as_ptr()) }
            }
            _ => panic!("Unsupported file format"),
        }
    }

    pub fn is_ready(&self) -> bool {
        unsafe { IsMusicReady(self.clone()) }
    }

    pub fn play(&self) {
        unsafe {
            PlayMusicStream(self.clone());
        }
    }

    pub fn is_playing(&self) -> bool {
        unsafe { IsMusicStreamPlaying(self.clone()) }
    }

    pub fn update(&self) {
        unsafe {
            UpdateMusicStream(self.clone());
        }
    }

    pub fn stop(&self) {
        unsafe {
            StopMusicStream(self.clone());
        }
    }

    pub fn pause(&self) {
        unsafe {
            PauseMusicStream(self.clone());
        }
    }

    pub fn resume(&self) {
        unsafe {
            ResumeMusicStream(self.clone());
        }
    }

    pub fn seek(&self, position: f32) {
        unsafe {
            SeekMusicStream(self.clone(), position as c_float);
        }
    }

    pub fn set_volume(&self, volume: f32) {
        unsafe {
            SetMusicVolume(self.clone(), volume as c_float);
        }
    }

    pub fn set_pitch(&self, pitch: f32) {
        unsafe {
            SetMusicPitch(self.clone(), pitch as c_float);
        }
    }

    pub fn set_pan(&self, pan: f32) {
        unsafe {
            SetMusicPan(self.clone(), pan as c_float);
        }
    }

    pub fn duration(&self) -> Duration {
        let dur = unsafe { GetMusicTimeLength(self.clone()) };
        Duration::from_secs_f32(dur as f32)
    }

    pub fn time_played(&self) -> Duration {
        let dur = unsafe { GetMusicTimePlayed(self.clone()) };
        Duration::from_secs_f32(dur as f32)
    }
}

impl Drop for Music {
    fn drop(&mut self) {
        unsafe {
            UnloadMusicStream(self.clone());
        }
    }
}

unsafe extern "C" {
    fn InitAudioDevice();
    fn CloseAudioDevice();
    fn IsAudioDeviceReady() -> bool;
    fn SetMasterVolume(volume: c_float);
    fn GetMasterVolume() -> c_float;

    fn LoadWave(fileName: *const c_char) -> Wave;
    fn IsWaveReady(wave: Wave) -> bool;
    fn LoadSound(fileName: *const c_char) -> Sound;
    fn LoadSoundFromWave(wave: Wave) -> Sound;
    fn IsSoundReady(sound: Sound) -> bool;
    fn UnloadWave(wave: Wave);
    fn UnloadSound(sound: Sound);
    fn ExportWave(wave: Wave, fileName: *const c_char) -> bool;

    fn PlaySound(sound: Sound);
    fn StopSound(sound: Sound);
    fn PauseSound(sound: Sound);
    fn ResumeSound(sound: Sound);
    fn IsSoundPlaying(sound: Sound) -> bool;
    fn SetSoundVolume(sound: Sound, volume: c_float);
    fn SetSoundPitch(sound: Sound, pitch: c_float);
    fn SetSoundPan(sound: Sound, pan: c_float);
    fn WaveCrop(wave: *mut Wave, initSample: c_int, finalSample: c_int);
    fn WaveFormat(wave: *mut Wave, sampleRate: c_int, sampleSize: c_int, channels: c_int);

    fn LoadMusicStream(fileName: *const c_char) -> Music;
    fn IsMusicReady(music: Music) -> bool;
    fn UnloadMusicStream(music: Music);
    fn PlayMusicStream(music: Music);
    fn IsMusicStreamPlaying(music: Music) -> bool;
    fn UpdateMusicStream(music: Music);
    fn StopMusicStream(music: Music);
    fn PauseMusicStream(music: Music);
    fn ResumeMusicStream(music: Music);
    fn SeekMusicStream(music: Music, position: c_float);
    fn SetMusicVolume(music: Music, volume: c_float);
    fn SetMusicPitch(music: Music, pitch: c_float);
    fn SetMusicPan(music: Music, pan: c_float);
    fn GetMusicTimeLength(music: Music) -> c_float;
    fn GetMusicTimePlayed(music: Music) -> c_float;
}
