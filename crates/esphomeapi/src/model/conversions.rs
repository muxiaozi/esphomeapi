use std::collections::HashMap;

use super::{entity_state::EntityState, EntityInfo};
use crate::{proto::api, utils::Options as _, Result};

lazy_static::lazy_static! {
    pub static ref LIST_ENTITIES_SERVICES_RESPONSE_TYPES: HashMap<u32, fn(&[u8]) -> Result<EntityInfo>> = {
        let mut m = HashMap::new();
        m.insert(api::ListEntitiesAlarmControlPanelResponse::get_option_id(), EntityInfo::parse_alarm_control_panel as fn(&[u8]) -> Result<EntityInfo>);
        m.insert(api::ListEntitiesBinarySensorResponse::get_option_id(), EntityInfo::parse_binary_sensor);
        m.insert(api::ListEntitiesButtonResponse::get_option_id(), EntityInfo::parse_button);
        m.insert(api::ListEntitiesCameraResponse::get_option_id(), EntityInfo::parse_camera);
        m.insert(api::ListEntitiesClimateResponse::get_option_id(), EntityInfo::parse_climate);
        m.insert(api::ListEntitiesCoverResponse::get_option_id(), EntityInfo::parse_cover);
        m.insert(api::ListEntitiesDateResponse::get_option_id(), EntityInfo::parse_date);
        m.insert(api::ListEntitiesDateTimeResponse::get_option_id(), EntityInfo::parse_date_time);
        m.insert(api::ListEntitiesEventResponse::get_option_id(), EntityInfo::parse_event);
        m.insert(api::ListEntitiesFanResponse::get_option_id(), EntityInfo::parse_fan);
        m.insert(api::ListEntitiesLightResponse::get_option_id(), EntityInfo::parse_light);
        m.insert(api::ListEntitiesLockResponse::get_option_id(), EntityInfo::parse_lock);
        m.insert(api::ListEntitiesMediaPlayerResponse::get_option_id(), EntityInfo::parse_media_player);
        m.insert(api::ListEntitiesNumberResponse::get_option_id(), EntityInfo::parse_number);
        m.insert(api::ListEntitiesSelectResponse::get_option_id(), EntityInfo::parse_select);
        m.insert(api::ListEntitiesSensorResponse::get_option_id(), EntityInfo::parse_sensor);
        m.insert(api::ListEntitiesSwitchResponse::get_option_id(), EntityInfo::parse_switch);
        m.insert(api::ListEntitiesTextResponse::get_option_id(), EntityInfo::parse_text);
        m.insert(api::ListEntitiesTextSensorResponse::get_option_id(), EntityInfo::parse_text_sensor);
        m.insert(api::ListEntitiesTimeResponse::get_option_id(), EntityInfo::parse_time);
        m.insert(api::ListEntitiesUpdateResponse::get_option_id(), EntityInfo::parse_update);
        m.insert(api::ListEntitiesValveResponse::get_option_id(), EntityInfo::parse_valve);
        m
    };

    pub static ref SUBCRIBE_STATES_RESPONSE_TYPES: HashMap<u32, fn(&[u8]) -> Result<EntityState>> = {
        let mut m = HashMap::new();
        m.insert(api::AlarmControlPanelStateResponse::get_option_id(), EntityState::parse_alarm_control_panel as fn(&[u8]) -> Result<EntityState>);
        m.insert(api::BinarySensorStateResponse::get_option_id(), EntityState::parse_binary_sensor);
        m.insert(api::ClimateStateResponse::get_option_id(), EntityState::parse_climate);
        m.insert(api::CoverStateResponse::get_option_id(), EntityState::parse_cover);
        m.insert(api::DateStateResponse::get_option_id(), EntityState::parse_date);
        m.insert(api::DateTimeStateResponse::get_option_id(), EntityState::parse_date_time);
        m.insert(api::EventResponse::get_option_id(), EntityState::parse_event);
        m.insert(api::FanStateResponse::get_option_id(), EntityState::parse_fan);
        m.insert(api::LightStateResponse::get_option_id(), EntityState::parse_light);
        m.insert(api::LockStateResponse::get_option_id(), EntityState::parse_lock);
        m.insert(api::MediaPlayerStateResponse::get_option_id(), EntityState::parse_media_player);
        m.insert(api::NumberStateResponse::get_option_id(), EntityState::parse_number);
        m.insert(api::SelectStateResponse::get_option_id(), EntityState::parse_select);
        m.insert(api::SensorStateResponse::get_option_id(), EntityState::parse_sensor);
        m.insert(api::SwitchStateResponse::get_option_id(), EntityState::parse_switch);
        m.insert(api::TextStateResponse::get_option_id(), EntityState::parse_text);
        m.insert(api::TextSensorStateResponse::get_option_id(), EntityState::parse_text_sensor);
        m.insert(api::TimeStateResponse::get_option_id(), EntityState::parse_time);
        m.insert(api::UpdateStateResponse::get_option_id(), EntityState::parse_update);
        m.insert(api::ValveStateResponse::get_option_id(), EntityState::parse_valve);
        m
    };
}
