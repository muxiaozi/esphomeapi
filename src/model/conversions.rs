use std::collections::HashMap;

use super::{services, Result, ServiceEntityInfo};
use crate::{proto::api, utils::Options as _};

lazy_static::lazy_static! {
    static ref LIST_ENTITIES_SERVICES_RESPONSE_TYPES: HashMap<u32, fn(&[u8]) -> Result<ServiceEntityInfo>> = {
        let mut m = HashMap::new();
        m.insert(api::ListEntitiesAlarmControlPanelResponse::get_option_id(), ServiceEntityInfo::parse_alarm_control_panel as fn(&[u8]) -> Result<ServiceEntityInfo>);
        m.insert(api::ListEntitiesBinarySensorResponse::get_option_id(), ServiceEntityInfo::parse_binary_sensor);
        m.insert(api::ListEntitiesButtonResponse::get_option_id(), ServiceEntityInfo::parse_button);
        m.insert(api::ListEntitiesCameraResponse::get_option_id(), ServiceEntityInfo::parse_camera);
        m.insert(api::ListEntitiesClimateResponse::get_option_id(), ServiceEntityInfo::parse_climate);
        m.insert(api::ListEntitiesCoverResponse::get_option_id(), ServiceEntityInfo::parse_cover);
        m.insert(api::ListEntitiesDateResponse::get_option_id(), ServiceEntityInfo::parse_date);
        m.insert(api::ListEntitiesDateTimeResponse::get_option_id(), ServiceEntityInfo::parse_date_time);
        m.insert(api::ListEntitiesEventResponse::get_option_id(), ServiceEntityInfo::parse_event);
        m.insert(api::ListEntitiesFanResponse::get_option_id(), ServiceEntityInfo::parse_fan);
        m.insert(api::ListEntitiesLightResponse::get_option_id(), ServiceEntityInfo::parse_light);
        m.insert(api::ListEntitiesLockResponse::get_option_id(), ServiceEntityInfo::parse_lock);
        m.insert(api::ListEntitiesMediaPlayerResponse::get_option_id(), ServiceEntityInfo::parse_media_player);
        m.insert(api::ListEntitiesNumberResponse::get_option_id(), ServiceEntityInfo::parse_number);
        m.insert(api::ListEntitiesSelectResponse::get_option_id(), ServiceEntityInfo::parse_select);
        m.insert(api::ListEntitiesSensorResponse::get_option_id(), ServiceEntityInfo::parse_sensor);
        m.insert(api::ListEntitiesSwitchResponse::get_option_id(), ServiceEntityInfo::parse_switch);
        m.insert(api::ListEntitiesTextResponse::get_option_id(), ServiceEntityInfo::parse_text);
        m.insert(api::ListEntitiesTextSensorResponse::get_option_id(), ServiceEntityInfo::parse_text_sensor);
        m.insert(api::ListEntitiesTimeResponse::get_option_id(), ServiceEntityInfo::parse_time);
        m.insert(api::ListEntitiesValveResponse::get_option_id(), ServiceEntityInfo::parse_valve);
        m
    };
}
