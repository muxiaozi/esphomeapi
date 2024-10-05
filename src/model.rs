use std::collections::HashMap;

use enumflags2::bitflags;

use crate::proto;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct APIVersion {
  major: u8,
  minor: u8,
}

impl APIVersion {
  pub fn new(major: u8, minor: u8) -> APIVersion {
    APIVersion { major, minor }
  }
}

enum BluetoothProxyFeature {
  PassiveScan = 1 << 0,
  ActiveConnections = 1 << 1,
  RemoteCaching = 1 << 2,
  Pairing = 1 << 3,
  CacheClearing = 1 << 4,
  RawAdvertisements = 1 << 5,
}

enum BluetoothProxySubscriptionFlag {
  RawAdvertisements = 1 << 0,
}

enum VoiceAssistantFeature {
  VoiceAssistant = 1 << 0,
  Speaker = 1 << 1,
  APIAudio = 1 << 2,
  Timers = 1 << 3,
  Announce = 1 << 4,
}

enum VoiceAssistantSubscriptionFlag {
  APIAudio = 1 << 0,
}

struct DeviceInfo {
  uses_password: bool,
  name: String,
  friendly_name: String,
  mac_address: String,
  compilation_time: String,
  model: String,
  manufacturer: String,
  has_deep_sleep: bool,
  esphome_version: String,
  project_name: String,
  project_version: String,
  webserver_port: u16,
  legacy_voice_assistant_version: u8,
  voice_assistant_feature_flags: u8,
  legacy_bluetooth_proxy_version: u8,
  bluetooth_proxy_feature_flags: u8,
  suggested_area: String,
}

impl DeviceInfo {
  fn new() -> DeviceInfo {
    DeviceInfo {
      uses_password: false,
      name: String::new(),
      friendly_name: String::new(),
      mac_address: String::new(),
      compilation_time: String::new(),
      model: String::new(),
      manufacturer: String::new(),
      has_deep_sleep: false,
      esphome_version: String::new(),
      project_name: String::new(),
      project_version: String::new(),
      webserver_port: 0,
      legacy_voice_assistant_version: 0,
      voice_assistant_feature_flags: 0,
      legacy_bluetooth_proxy_version: 0,
      bluetooth_proxy_feature_flags: 0,
      suggested_area: String::new(),
    }
  }

  pub fn bluetooth_proxy_feature_flags_compat(&self, api_version: APIVersion) -> u8 {
    if api_version < APIVersion::new(1, 9) {
      let mut flags = 0;
      if self.legacy_bluetooth_proxy_version >= 1 {
        flags |= BluetoothProxyFeature::PassiveScan as u8;
      }
      if self.legacy_bluetooth_proxy_version >= 2 {
        flags |= BluetoothProxyFeature::ActiveConnections as u8;
      }
      if self.legacy_bluetooth_proxy_version >= 3 {
        flags |= BluetoothProxyFeature::RemoteCaching as u8;
      }
      if self.legacy_bluetooth_proxy_version >= 4 {
        flags |= BluetoothProxyFeature::Pairing as u8;
      }
      if self.legacy_bluetooth_proxy_version >= 5 {
        flags |= BluetoothProxyFeature::CacheClearing as u8;
      }
      return flags;
    }
    return self.bluetooth_proxy_feature_flags;
  }

  pub fn voice_assistant_feature_flags_compat(&self, api_version: APIVersion) -> u8 {
    if api_version < APIVersion::new(1, 10) {
      let mut flags = 0;
      if self.legacy_voice_assistant_version >= 1 {
        flags |= VoiceAssistantFeature::VoiceAssistant as u8;
      }
      if self.legacy_voice_assistant_version >= 2 {
        flags |= VoiceAssistantFeature::Speaker as u8;
      }
      return flags;
    }
    return self.voice_assistant_feature_flags;
  }
}

enum EntityCategory {
  None = 0,
  Config,
  Diagnostic,
}

struct EntityInfo {
  object_id: String,
  key: u8,
  name: String,
  unique_id: String,
  disabled_by_default: bool,
  icon: String,
  enitity_category: Option<EntityCategory>,
}

struct EntityState {
  key: u8,
}

// ==================== BINARY SENSOR ====================
struct BinarySensorInfo {
  entity_info: EntityInfo,
  device_class: String,
  is_status_binary_sensor: bool,
}

struct BinarySensorState {
  entity_state: EntityState,
  state: bool,
  missing_state: bool,
}

// ==================== COVER ====================

struct CoverInfo {
  entity_info: EntityInfo,
  assumed_state: bool,
  supports_stop: bool,
  supports_position: bool,
  supports_tilt: bool,
  device_class: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
enum LegacyCoverState {
  Open = 0,
  Closed,
}

enum LegacyCoverCommand {
  Open = 0,
  Close,
  Stop,
}

enum CoverOperation {
  Idle = 0,
  Opening,
  Closing,
}

struct CoverState {
  entity_state: EntityState,
  legacy_state: Option<LegacyCoverState>,
  position: f32,
  tilt: f32,
  current_operation: Option<CoverOperation>,
}

impl CoverState {
  pub fn is_closed(&self, api_version: APIVersion) -> bool {
    if api_version < APIVersion::new(1, 1) {
      if let Some(legacy_state) = self.legacy_state {
        return legacy_state == LegacyCoverState::Closed;
      }
    }
    return self.position == 0.0;
  }
}

// ==================== EVENT ====================
struct EventInfo {
  entity_info: EntityInfo,
  device_class: String,
  event_types: Vec<String>,
}

struct Event {
  entity_state: EntityState,
  event_type: String,
}

// ==================== FAN ====================
struct FanInfo {
  entity_info: EntityInfo,
  supports_oscillation: bool,
  supports_speed: bool,
  supports_direction: bool,
  supported_speed_levels: u8,
  supported_preset_modes: Vec<String>,
}

enum FanSpeed {
  Low = 0,
  Medium,
  High,
}

enum FanDirection {
  Forward = 0,
  Reverse,
}

struct FanState {
  entity_state: EntityState,
  oscillating: bool,
  speed: Option<FanSpeed>,
  speed_level: u8,
  direction: Option<FanDirection>,
  preset_mode: String,
}

// ==================== LIGHT ====================
#[bitflags]
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
enum LightColorCapability {
  OnOff = 1 << 0,
  Brightness = 1 << 1,
  White = 1 << 2,
  ColorTemperature = 1 << 3,
  ColdWarmWhite = 1 << 4,
  RGB = 1 << 5,
}

struct LightInfo {
  entity_info: EntityInfo,
  supported_color_modes: Vec<u8>,
  min_mireds: f32,
  max_mireds: f32,
  effects: Vec<String>,

  // deprecated, do not use
  legacy_supports_brightness: bool,
  legacy_supports_rgb: bool,
  legacy_supports_white_value: bool,
  legacy_supports_color_temperature: bool,
}

impl LightInfo {
  pub fn supported_color_modes_compat(&self, api_version: APIVersion) -> Vec<u8> {
    if api_version < APIVersion::new(1, 6) {
      let key = (
        self.legacy_supports_brightness,
        self.legacy_supports_rgb,
        self.legacy_supports_white_value,
        self.legacy_supports_color_temperature,
      );

      let legacy_mode = match key {
        (false, false, false, false) => LightColorCapability::OnOff as u8,
        (true, false, false, false) => {
          (LightColorCapability::OnOff | LightColorCapability::Brightness).bits()
        }
        (true, false, false, true) => (LightColorCapability::OnOff
          | LightColorCapability::Brightness
          | LightColorCapability::ColorTemperature)
          .bits(),
        (true, true, false, false) => (LightColorCapability::OnOff
          | LightColorCapability::Brightness
          | LightColorCapability::RGB)
          .bits(),
        (true, true, true, false) => (LightColorCapability::OnOff
          | LightColorCapability::Brightness
          | LightColorCapability::RGB
          | LightColorCapability::White)
          .bits(),
        (true, true, false, true) => (LightColorCapability::OnOff
          | LightColorCapability::Brightness
          | LightColorCapability::RGB
          | LightColorCapability::ColorTemperature)
          .bits(),
        (true, true, true, true) => (LightColorCapability::OnOff
          | LightColorCapability::Brightness
          | LightColorCapability::RGB
          | LightColorCapability::White
          | LightColorCapability::ColorTemperature)
          .bits(),
        _ => LightColorCapability::OnOff as u8,
      };

      return vec![legacy_mode];
    }
    return self.supported_color_modes.clone();
  }
}

struct LightState {
  entity_state: EntityState,
  state: bool,
  brightness: f32,
  color_mode: u8,
  color_brightness: f32,
  red: f32,
  green: f32,
  blue: f32,
  white: f32,
  color_temperature: f32,
  cold_white: f32,
  warm_white: f32,
  effect: String,
}

// ==================== SENSOR ====================

enum SensorStateClass {
  None = 0,
  Measurement,
  TotalIncreasing,
  Total,
}

enum LastResetType {
  None = 0,
  Never,
  Auto,
}

struct SensorInfo {
  entity_info: EntityInfo,
  device_class: String,
  unit_of_measurement: String,
  accuracy_decimals: u8,
  force_update: bool,
  state_class: Option<SensorStateClass>,
  last_reset_type: Option<LastResetType>,
}

struct SensorState {
  entity_state: EntityState,
  state: f32,
  missing_state: bool,
}

// ==================== SWITCH ====================
struct SwitchInfo {
  entity_info: EntityInfo,
  assumed_state: bool,
  device_class: String,
}

struct SwitchState {
  entity_state: EntityState,
  state: bool,
}

// ==================== TEXT SENSOR ====================
struct TextSensorInfo {
  entity_info: EntityInfo,
  device_class: String,
}

struct TextSensorState {
  entity_state: EntityState,
  state: String,
  missing_state: bool,
}

// ==================== CAMERA ====================
struct CameraInfo {
  entity_info: EntityInfo,
}

struct CameraState {
  entity_state: EntityState,
  data: Vec<u8>,
}

// ==================== CLIMATE ====================

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
enum ClimateMode {
  Off = 0,
  HeatCool,
  Cool,
  Heat,
  FanOnly,
  Dry,
  Auto,
}

enum ClimateFanMode {
  On = 0,
  Off,
  Auto,
  Low,
  Medium,
  High,
  Middle,
  Focus,
  Diffuse,
  Quiet,
}

enum ClimateSwingMode {
  Off = 0,
  Both,
  Vertical,
  Horizontal,
}

enum ClimateAction {
  Off = 0,
  Cooling,
  Heating,
  Idle,
  Drying,
  Fan,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
enum ClimatePreset {
  None = 0,
  Home,
  Away,
  Boost,
  Comfort,
  Eco,
  Sleep,
  Activity,
}

struct ClimateInfo {
  entity_info: EntityInfo,
  supports_current_temperature: bool,
  supports_two_point_target_temperature: bool,
  supported_modes: Vec<ClimateMode>,
  visual_min_temperature: f32,
  visual_max_temperature: f32,
  visual_target_temperature_step: f32,
  visual_current_temperature_step: f32,
  legacy_supports_away: bool,
  supports_action: bool,
  supported_fan_modes: Vec<ClimateFanMode>,
  supported_swing_modes: Vec<ClimateSwingMode>,
  supported_custom_fan_modes: Vec<String>,
  supported_presets: Vec<ClimatePreset>,
  supported_custom_presets: Vec<String>,
  supports_current_humidity: bool,
  supports_target_humidity: bool,
  visual_min_humidity: f32,
  visual_max_humidity: f32,
}

impl ClimateInfo {
  pub fn supported_presets_compat(&self, api_version: APIVersion) -> Vec<ClimatePreset> {
    if api_version < APIVersion::new(1, 5) {
      if self.legacy_supports_away {
        return vec![ClimatePreset::Home, ClimatePreset::Away];
      }
      return vec![];
    }
    return self.supported_presets.clone();
  }
}

struct ClimateState {
  entity_state: EntityState,
  mode: Option<ClimateMode>,
  action: Option<ClimateAction>,
  current_temperature: f32,
  target_temperature: f32,
  target_temperature_low: f32,
  target_temperature_high: f32,
  legacy_away: bool,
  fan_mode: Option<ClimateFanMode>,
  swing_mode: Option<ClimateSwingMode>,
  custom_fan_mode: String,
  preset: Option<ClimatePreset>,
  custom_preset: String,
  current_humidity: f32,
  target_humidity: f32,
}

impl ClimateState {
  pub fn preset_compat(&self, api_version: APIVersion) -> Option<ClimatePreset> {
    if api_version < APIVersion::new(1, 5) {
      if self.legacy_away {
        return Some(ClimatePreset::Away);
      }
      return Some(ClimatePreset::Home);
    }
    return self.preset;
  }
}

// ==================== NUMBER ====================

enum NumberMode {
  Auto = 0,
  Box,
  Slider,
}

struct NumberInfo {
  entity_info: EntityInfo,
  min_value: f32,
  max_value: f32,
  step: f32,
  unit_of_measurement: String,
  mode: Option<NumberMode>,
  device_class: String,
}

struct NumberState {
  entity_state: EntityState,
  state: f32,
  missing_state: bool,
}

// ==================== DATETIME DATE ====================

struct DateInfo {
  entity_info: EntityInfo,
}

struct DateState {
  entity_state: EntityState,
  missing_state: bool,
  year: u16,
  month: u8,
  day: u8,
}

// ==================== DATETIME TIME ====================
struct TimeInfo {
  entity_info: EntityInfo,
}

struct TimeState {
  entity_state: EntityState,
  missing_state: bool,
  hour: u8,
  minute: u8,
  second: u8,
}

// ==================== DATETIME DATETIME ====================

struct DateTimeInfo {
  entity_info: EntityInfo,
}

struct DateTimeState {
  entity_state: EntityState,
  missing_state: bool,
  epoch_seconds: u32,
}

// ==================== SELECT ====================

struct SelectInfo {
  entity_info: EntityInfo,
  options: Vec<String>,
}

struct SelectState {
  entity_state: EntityState,
  state: String,
  missing_state: bool,
}

// ==================== SIREN ====================

struct SirenInfo {
  entity_info: EntityInfo,
  tones: Vec<String>,
  supports_volume: bool,
  supports_duration: bool,
}

struct SirenState {
  entity_state: EntityState,
  state: bool,
}

// ==================== BUTTON ====================

struct ButtonInfo {
  entity_info: EntityInfo,
  device_class: String,
}

// ==================== LOCK ====================

enum LockState {
  None = 0,
  Locked,
  Unlocked,
  Jammed,
  Locking,
  Unlocking,
}

enum LockCommand {
  Unlock = 0,
  Lock,
  Open,
}

struct LockInfo {
  entity_info: EntityInfo,
  supports_open: bool,
  assumed_state: bool,

  requires_code: bool,
  code_format: String,
}

struct LockEntityState {
  entity_state: EntityState,
  state: Option<LockState>,
}

// ==================== VALVE ====================

struct ValveInfo {
  entity_info: EntityInfo,
  device_class: String,
  assumed_state: bool,
  supports_stop: bool,
  supports_position: bool,
}

enum ValveOperation {
  Idle = 0,
  Opening,
  Closing,
}

struct ValveState {
  entity_state: EntityState,
  position: f32,
  current_operation: Option<ValveOperation>,
}

// ==================== MEDIA PLAYER ====================

enum MediaPlayerState {
  None = 0,
  Idle,
  Playing,
  Paused,
}

enum MediaPlayerCommand {
  Play = 0,
  Pause,
  Stop,
  Mute,
  Unmute,
}

enum MediaPlayerFormatPurpose {
  Default = 0,
  Announcement,
}

struct MediaPlayerSupportedFormat {
  format: String,
  sample_rate: u32,
  num_channels: u8,
  purpose: Option<MediaPlayerFormatPurpose>,
  sample_bytes: u8,
}

struct MediaPlayerInfo {
  entity_info: EntityInfo,
  supports_pause: bool,
  supported_formats: Vec<MediaPlayerSupportedFormat>,
}

struct MediaPlayerEntityState {
  entity_state: EntityState,
  state: Option<MediaPlayerState>,
  volume: f32,
  muted: bool,
}

// ==================== ALARM CONTROL PANEL ====================

enum AlarmControlPanelState {
  Disarmed = 0,
  ArmedHome,
  ArmedAway,
  ArmedNight,
  ArmedVacation,
  ArmedCustomBypass,
  Pending,
  Arming,
  Disarming,
  Triggered,
}

enum AlarmControlPanelCommand {
  Disarm = 0,
  ArmHome,
  ArmAway,
  ArmNight,
  ArmVacation,
  ArmCustomBypass,
  Trigger,
}

struct AlarmControlPanelInfo {
  entity_info: EntityInfo,
  supported_features: u8,
  requires_code: bool,
  requires_code_to_arm: bool,
}

struct AlarmControlPanelEntityState {
  entity_state: EntityState,
  state: Option<AlarmControlPanelState>,
}

// ==================== TEXT ====================
enum TextMode {
  Text = 0,
  Password,
}

struct TextInfo {
  entity_info: EntityInfo,
  min_length: u8,
  max_length: u8,
  pattern: String,
  mode: Option<TextMode>,
}

struct TextState {
  entity_state: EntityState,
  state: String,
  missing_state: bool,
}

// ==================== UPDATE ====================
enum UpdateCommand {
  None = 0,
  Install,
  Check,
}

struct UpdateInfo {
  entity_info: EntityInfo,
  device_class: String,
}

struct UpdateState {
  entity_state: EntityState,
  missing_state: bool,
  in_progress: bool,
  has_progress: bool,
  progress: f32,
  current_version: String,
  latest_version: String,
  title: String,
  release_summary: String,
  release_url: String,
}

// ==================== USER-DEFINED SERVICES ====================
struct HomeassistantServiceCall {
  service: String,
  is_event: bool,
  data: HashMap<String, String>,
  data_template: HashMap<String, String>,
  variables: HashMap<String, String>,
}

enum UserServiceArgType {
  Bool = 0,
  Int,
  Float,
  String,
  BoolArray,
  IntArray,
  FloatArray,
  StringArray,
}

struct UserServiceArg {
  name: String,
  arg_type: UserServiceArgType,
}

struct UserService {
  name: String,
  key: u8,
  args: Vec<UserServiceArg>,
}

// ==================== BLUETOOTH ====================

fn uuid_convert(uuid: String) -> String {
  let mut uuid = uuid.to_lowercase();
  if uuid.len() < 8 {
    uuid = format!("0000{}-0000-1000-8000-00805f9b34fb", uuid[2..].to_string());
  }
  return uuid;
}

struct BluetoothLEAdvertisement {
  address: u64,
  rssi: i32,
  address_type: u32,
  name: String,
  service_uuids: Vec<String>,
  service_data: HashMap<String, Vec<u8>>,
  manufacturer_data: HashMap<u16, Vec<u8>>,
}

impl BluetoothLEAdvertisement {
  pub fn from_pb(data: proto::api::BluetoothLEAdvertisementResponse) -> Self {
    let mut manufacturer_data: HashMap<u16, Vec<u8>> = HashMap::new();
    let mut service_data: HashMap<String, Vec<u8>> = HashMap::new();
    let mut service_uuids: Vec<String> = Vec::new();

    let raw_manufacturer_data = data.manufacturer_data;
    if !raw_manufacturer_data.is_empty() {
      if !raw_manufacturer_data[0].data.is_empty() {
        raw_manufacturer_data.iter().for_each(|item| {
          manufacturer_data.insert(item.uuid.parse().unwrap(), item.data.clone());
        });
      } else {
        // legacy data
        raw_manufacturer_data.iter().for_each(|item| {
          manufacturer_data.insert(
            item.uuid.parse().unwrap(),
            item
              .legacy_data
              .iter()
              .flat_map(|&num| num.to_le_bytes().to_vec())
              .collect(),
          );
        });
      }
    }

    let raw_service_data = data.service_data;
    if !raw_service_data.is_empty() {
      if !raw_service_data[0].data.is_empty() {
        raw_service_data.iter().for_each(|item| {
          service_data.insert(uuid_convert(item.uuid.clone()), item.data.clone());
        });
      } else {
        // legacy data
        raw_service_data.iter().for_each(|item| {
          service_data.insert(
            uuid_convert(item.uuid.clone()),
            item
              .legacy_data
              .iter()
              .flat_map(|&num| num.to_le_bytes().to_vec())
              .collect(),
          );
        });
      }
    }

    let raw_service_uuids = data.service_uuids;
    if !raw_service_uuids.is_empty() {
      service_uuids.extend(
        raw_service_uuids
          .iter()
          .map(|uuid| uuid_convert(uuid.clone())),
      );
    }

    Self {
      address: data.address,
      rssi: data.rssi,
      address_type: data.address_type,
      name: data.name, // TODO: check if correct, UTF-8 conversion might be needed
      service_uuids,
      service_data,
      manufacturer_data,
    }
  }
}

struct BluetoothDeviceConnection {
  address: u64,
  connected: bool,
  mtu: u16,
  error: u8,
}

struct BluetoothDevicePairing {
  address: u64,
  paired: bool,
  error: u8,
}

struct BluetoothDeviceUnpairing {
  address: u64,
  success: bool,
  error: u8,
}

struct BluetoothDeviceClearCache {
  address: u64,
  success: bool,
  error: u8,
}

struct BluetoothGATTRead {
  address: u64,
  handle: u16,
  data: Vec<u8>,
}

struct BluetoothGATTDescriptor {
  uuid: String,
  handle: u16,
}

struct BluetoothGATTCharacteristic {
  uuid: String,
  handle: u16,
  properties: u8,
  descriptors: Vec<BluetoothGATTDescriptor>,
}

struct BluetoothGATTService {
  uuid: String,
  handle: u16,
  characteristics: Vec<BluetoothGATTCharacteristic>,
}

struct BluetoothGATTServices {
  address: u64,
  services: Vec<BluetoothGATTService>,
}

struct ESPHomeBluetoothGATTServices {
  address: u64,
  services: Vec<BluetoothGATTService>,
}

struct BluetoothConnectionsFree {
  free: u8,
  limit: u8,
}

struct BluetoothGATTError {
  address: u64,
  handle: u16,
  error: u8,
}

enum BluetoothDeviceRequestType {
  Connect = 0,
  Disconnect,
  Pair,
  Unpair,
  ConnectV3WithCache,
  ConnectV3WithoutCache,
  ClearCache,
}

enum VoiceAssistantCommandFlag {
  UseVAD = 1 << 0,
  UseWakeWord = 1 << 1,
}

struct VoiceAssistantAudioSettings {
  noise_suppression_level: u8,
  auto_gain: u8,
  volume_multiplier: f32,
}

struct VoiceAssistantCommand {
  start: bool,
  conversation_id: String,
  flags: u8,
  audio_settings: Vec<VoiceAssistantAudioSettings>,
  wake_word_phrase: String,
}

struct VoiceAssistantAudioData {
  data: Vec<u8>,
  end: bool,
}

struct VoiceAssistantAnnounceFinished {
  success: bool,
}

struct VoiceAssistantWakeWord {
  id: String,
  wake_word: String,
  trained_languages: Vec<String>,
}

struct VoiceAssistantConfigurationResponse {
  available_wake_words: Vec<VoiceAssistantWakeWord>,
  active_wake_words: Vec<String>,
  max_active_wake_words: u8,
}

struct VoiceAssistantConfigurationRequest {}

struct VoiceAssistantSetConfiguration {
  active_wake_words: Vec<u8>,
}

enum LogLevel {
  None = 0,
  Error,
  Warn,
  Info,
  Config,
  Debug,
  Verbose,
  VeryVerbose,
}

enum VoiceAssistantEventType {
  Error = 0,
  RunStart,
  RunEnd,
  STTStart,
  STTEnd,
  IntentStart,
  IntentEnd,
  TTSStart,
  TTSEnd,
  WakeWordStart,
  WakeWordEnd,
  SSTVADStart,
  SSTVADEnd,
  TTSStreamStart = 98,
  TTSStreamEnd = 99,
}

enum VoiceAssistantTimerEventType {
  TimerStarted = 0,
  TimerUpdated,
  TimerCancelled,
  TimerFinished,
}
