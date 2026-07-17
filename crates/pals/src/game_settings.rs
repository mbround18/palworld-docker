use env_parse::env_parse;
use gsm_serde::serde_ini::{IniHeader, to_string, to_string_compact};
use gsm_shared::fetch_var;
use ini_derive::IniSerialize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::create_dir_all;
use std::path::Path;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Preset {
    Casual,
    Normal,
    Hard,
}

#[derive(Debug, Clone, Serialize, Deserialize, IniSerialize, Default)]
#[INIHeader(name = "/Script/Pal.PalGameWorldSettings")]
pub struct Settings {
    #[serde(rename = "OptionSettings")]
    option_settings: GameSettings,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct GameSettings {
    // Core gameplay rates
    #[serde(rename = "Difficulty")]
    pub difficulty: String,

    #[serde(rename = "RandomizerType")]
    pub randomizer_type: String,

    #[serde(rename = "RandomizerSeed")]
    pub randomizer_seed: String,

    #[serde(rename = "bIsRandomizerPalLevelRandom")]
    pub is_randomizer_pal_level_random: bool,

    #[serde(rename = "DayTimeSpeedRate")]
    pub day_time_speed_rate: f32,

    #[serde(rename = "NightTimeSpeedRate")]
    pub night_time_speed_rate: f32,

    #[serde(rename = "ExpRate")]
    pub exp_rate: f32,

    #[serde(rename = "PalCaptureRate")]
    pub pal_capture_rate: f32,

    #[serde(rename = "PalSpawnNumRate")]
    pub pal_spawn_num_rate: f32,

    #[serde(rename = "PalDamageRateAttack")]
    pub pal_damage_rate_attack: f32,

    #[serde(rename = "PalDamageRateDefense")]
    pub pal_damage_rate_defense: f32,

    #[serde(rename = "bAllowGlobalPalboxExport")]
    pub allow_global_palbox_export: bool,

    #[serde(rename = "bAllowGlobalPalboxImport")]
    pub allow_global_palbox_import: bool,

    #[serde(rename = "bCharacterRecreateInHardcore")]
    pub character_recreate_in_hardcore: bool,

    #[serde(rename = "PlayerDamageRateAttack")]
    pub player_damage_rate_attack: f32,

    #[serde(rename = "PlayerDamageRateDefense")]
    pub player_damage_rate_defense: f32,

    // NB: the engine's own ini key really does spell this "Decreace" — matches
    // the reference DefaultPalWorldSettings.ini's key, not the correct English.
    #[serde(rename = "PlayerStomachDecreaceRate")]
    pub player_stomach_decrease_rate: f32,

    #[serde(rename = "PlayerStaminaDecreaceRate")]
    pub player_stamina_decrease_rate: f32,

    #[serde(rename = "PlayerAutoHPRegeneRate")]
    pub player_auto_hp_regen_rate: f32,

    #[serde(rename = "PlayerAutoHpRegeneRateInSleep")]
    pub player_auto_hp_regen_rate_in_sleep: f32,

    #[serde(rename = "PalStomachDecreaceRate")]
    pub pal_stomach_decrease_rate: f32,

    #[serde(rename = "PalStaminaDecreaceRate")]
    pub pal_stamina_decrease_rate: f32,

    #[serde(rename = "PalAutoHPRegeneRate")]
    pub pal_auto_hp_regen_rate: f32,

    #[serde(rename = "PalAutoHpRegeneRateInSleep")]
    pub pal_auto_hp_regen_rate_in_sleep: f32,

    // Build and object settings
    #[serde(rename = "BuildObjectHpRate")]
    pub build_object_hp_rate: f32,

    #[serde(rename = "BuildObjectDamageRate")]
    pub build_object_damage_rate: f32,

    #[serde(rename = "BuildObjectDeteriorationDamageRate")]
    pub build_object_deterioration_damage_rate: f32,

    #[serde(rename = "CollectionDropRate")]
    pub collection_drop_rate: f32,

    #[serde(rename = "CollectionObjectHpRate")]
    pub collection_object_hp_rate: f32,

    #[serde(rename = "CollectionObjectRespawnSpeedRate")]
    pub collection_object_respawn_speed_rate: f32,

    #[serde(rename = "EnemyDropItemRate")]
    pub enemy_drop_item_rate: f32,

    // Death penalty and PvP settings
    #[serde(rename = "DeathPenalty")]
    pub death_penalty: String,

    #[serde(rename = "bEnablePlayerToPlayerDamage")]
    pub enable_pvp: bool,

    #[serde(rename = "bEnableFriendlyFire")]
    pub enable_friendly_fire: bool,

    #[serde(rename = "bEnableInvaderEnemy")]
    pub enable_invader_enemy: bool,

    #[serde(rename = "bActiveUNKO")]
    pub active_unko: bool,

    #[serde(rename = "bEnableAimAssistPad")]
    pub enable_aim_assist_pad: bool,

    #[serde(rename = "bEnableAimAssistKeyboard")]
    pub enable_aim_assist_keyboard: bool,

    // Drop and base camp settings
    #[serde(rename = "DropItemMaxNum")]
    pub drop_item_max_num: u32,

    #[serde(rename = "DropItemMaxNum_UNKO")]
    pub drop_item_max_num_unko: u32,

    #[serde(rename = "BaseCampMaxNum")]
    pub base_camp_max_num: u16,

    #[serde(rename = "BaseCampWorkerMaxNum")]
    pub base_camp_worker_max_num: u16,

    #[serde(rename = "DropItemAliveMaxHours")]
    pub drop_item_alive_max_hours: f32,

    // Guild and related settings
    #[serde(rename = "bAutoResetGuildNoOnlinePlayers")]
    pub auto_reset_guild_no_online_players: bool,

    #[serde(rename = "AutoResetGuildTimeNoOnlinePlayers")]
    pub auto_reset_guild_time_no_online_players: f32,

    #[serde(rename = "GuildPlayerMaxNum")]
    pub guild_player_max_num: u16,

    #[serde(rename = "BaseCampMaxNumInGuild")]
    pub base_camp_max_num_in_guild: u16,

    #[serde(rename = "PalEggDefaultHatchingTime")]
    pub pal_egg_default_hatching_time: f32,

    // Other gameplay rates
    #[serde(rename = "WorkSpeedRate")]
    pub work_speed_rate: f32,

    #[serde(rename = "AutoSaveSpan")]
    pub auto_save_span: f32,

    // Multiplayer and PvP modes
    #[serde(rename = "bIsMultiplay")]
    pub is_multiplay: bool,

    #[serde(rename = "bIsPvP")]
    pub is_pvp: bool,

    #[serde(rename = "bHardcore")]
    pub hardcore: bool,

    #[serde(rename = "bPalLost")]
    pub pal_lost: bool,

    #[serde(rename = "bCanPickupOtherGuildDeathPenaltyDrop")]
    pub can_pickup_other_guild_death_penalty_drop: bool,

    #[serde(rename = "bEnableNonLoginPenalty")]
    pub enable_non_login_penalty: bool,

    #[serde(rename = "bEnableFastTravel")]
    pub enable_fast_travel: bool,

    #[serde(rename = "bIsStartLocationSelectByMap")]
    pub is_start_location_select_by_map: bool,

    #[serde(rename = "bExistPlayerAfterLogout")]
    pub exist_player_after_logout: bool,

    #[serde(rename = "bEnableDefenseOtherGuildPlayer")]
    pub enable_defense_other_guild_player: bool,

    #[serde(rename = "bInvisibleOtherGuildBaseCampAreaFX")]
    pub invisible_other_guild_base_camp_area_fx: bool,

    #[serde(rename = "bBuildAreaLimit")]
    pub build_area_limit: bool,

    #[serde(rename = "ItemWeightRate")]
    pub item_weight_rate: f32,

    // Server limits and networking
    #[serde(rename = "CoopPlayerMaxNum")]
    pub coop_player_max_num: u16,

    #[serde(rename = "ServerPlayerMaxNum")]
    pub server_player_max_num: u16,

    #[serde(rename = "ServerName")]
    pub server_name: String,

    #[serde(rename = "ServerDescription")]
    pub server_description: String,

    #[serde(rename = "AdminPassword")]
    pub admin_password: String,

    #[serde(rename = "ServerPassword")]
    pub server_password: String,

    #[serde(rename = "PublicPort")]
    pub public_port: u16,

    #[serde(rename = "PublicIP")]
    pub public_ip: String,

    #[serde(rename = "RCONEnabled")]
    pub rcon_enabled: bool,

    #[serde(rename = "RCONPort")]
    pub rcon_port: u16,

    #[serde(rename = "bUseAuth")]
    pub use_auth: bool,

    #[serde(rename = "Region")]
    pub region: String,

    #[serde(rename = "BanListURL")]
    pub ban_list_url: String,

    #[serde(rename = "CrossplayPlatforms")]
    pub crossplay_platforms: String, // Default (Steam,Xbox,PS5,Mac)

    // REST API and additional networking
    #[serde(rename = "RESTAPIEnabled")]
    pub restapi_enabled: bool,

    #[serde(rename = "RESTAPIPort")]
    pub restapi_port: u16,

    #[serde(rename = "bShowPlayerList")]
    pub show_player_list: bool,

    #[serde(rename = "ChatPostLimitPerMinute")]
    pub chat_post_limit_per_minute: u16,

    #[serde(rename = "bIsUseBackupSaveData")]
    pub is_use_backup_save_data: bool,

    #[serde(rename = "LogFormatType")]
    pub log_format_type: String,

    #[serde(rename = "SupplyDropSpan")]
    pub supply_drop_span: f32,

    #[serde(rename = "EnablePredatorBossPal")]
    pub enable_predator_boss_pal: bool,

    #[serde(rename = "MaxBuildingLimitNum")]
    pub max_building_limit_num: u32,

    #[serde(rename = "ServerReplicatePawnCullDistance")]
    pub server_replicate_pawn_cull_distance: f32,
}

impl GameSettings {
    /// Constructs the base (Normal preset) configuration based on the golden INI.
    pub fn normal() -> Self {
        Self {
            difficulty: "None".to_owned(),
            randomizer_type: "None".to_owned(),
            randomizer_seed: String::new(),
            is_randomizer_pal_level_random: false,
            day_time_speed_rate: 1.0,
            night_time_speed_rate: 1.0,
            exp_rate: 1.0,
            pal_capture_rate: 1.0,
            pal_spawn_num_rate: 1.0,
            pal_damage_rate_attack: 1.0,
            pal_damage_rate_defense: 1.0,
            allow_global_palbox_export: false,
            allow_global_palbox_import: false,
            character_recreate_in_hardcore: false,
            player_damage_rate_attack: 1.0,
            player_damage_rate_defense: 1.0,
            player_stomach_decrease_rate: 1.0,
            player_stamina_decrease_rate: 1.0,
            player_auto_hp_regen_rate: 1.0,
            player_auto_hp_regen_rate_in_sleep: 1.0,
            pal_stomach_decrease_rate: 1.0,
            pal_stamina_decrease_rate: 1.0,
            pal_auto_hp_regen_rate: 1.0,
            pal_auto_hp_regen_rate_in_sleep: 1.0,
            build_object_hp_rate: 1.0,
            build_object_damage_rate: 1.0,
            build_object_deterioration_damage_rate: 1.0,
            collection_drop_rate: 1.0,
            collection_object_hp_rate: 1.0,
            collection_object_respawn_speed_rate: 1.0,
            enemy_drop_item_rate: 1.0,
            death_penalty: "All".to_owned(),
            enable_pvp: false,
            enable_friendly_fire: false,
            enable_invader_enemy: true,
            active_unko: false,
            enable_aim_assist_pad: true,
            enable_aim_assist_keyboard: false,
            drop_item_max_num: 3000,
            drop_item_max_num_unko: 100,
            base_camp_max_num: 128,
            base_camp_worker_max_num: 15,
            drop_item_alive_max_hours: 1.0,
            auto_reset_guild_no_online_players: false,
            auto_reset_guild_time_no_online_players: 72.0,
            guild_player_max_num: 20,
            base_camp_max_num_in_guild: 4,
            pal_egg_default_hatching_time: 72.0,
            work_speed_rate: 1.0,
            auto_save_span: 30.0,
            is_multiplay: false,
            is_pvp: false,
            hardcore: false,
            pal_lost: false,
            can_pickup_other_guild_death_penalty_drop: false,
            enable_non_login_penalty: true,
            enable_fast_travel: true,
            is_start_location_select_by_map: true,
            exist_player_after_logout: false,
            enable_defense_other_guild_player: false,
            invisible_other_guild_base_camp_area_fx: false,
            build_area_limit: false,
            item_weight_rate: 1.0,
            coop_player_max_num: 4,
            server_player_max_num: 32,
            server_name: "Default Palworld Server".to_owned(),
            server_description: String::new(),
            admin_password: String::new(),
            server_password: String::new(),
            public_port: 8211,
            public_ip: String::new(),
            rcon_enabled: false,
            rcon_port: 25575,
            use_auth: true,
            region: String::new(),
            ban_list_url: "https://api.palworldgame.com/api/banlist.txt".to_owned(),
            restapi_enabled: false,
            restapi_port: 8212,
            show_player_list: false,
            chat_post_limit_per_minute: 10,
            crossplay_platforms: "(Steam,Xbox,PS5,Mac)".to_owned(),
            is_use_backup_save_data: true,
            log_format_type: "Text".to_owned(),
            supply_drop_span: 180.0,
            enable_predator_boss_pal: true,
            max_building_limit_num: 0,
            server_replicate_pawn_cull_distance: 15000.0,
        }
    }

    /// Applies preset-specific overrides.
    pub fn apply_preset(&mut self, preset: Preset) {
        match preset {
            Preset::Casual => {
                self.day_time_speed_rate = 1.0;
                self.night_time_speed_rate = 1.0;
                self.exp_rate = 2.0;
                self.pal_capture_rate = 2.0;
                self.pal_spawn_num_rate = 1.0;
                self.pal_damage_rate_attack = 2.0;
                self.pal_damage_rate_defense = 0.5;
                self.player_damage_rate_attack = 2.0;
                self.player_damage_rate_defense = 0.5;
                self.player_stomach_decrease_rate = 0.3;
                self.player_stamina_decrease_rate = 0.3;
                self.player_auto_hp_regen_rate = 2.0;
                self.player_auto_hp_regen_rate_in_sleep = 2.0;
                self.build_object_damage_rate = 2.0;
                self.build_object_deterioration_damage_rate = 0.2;
                self.collection_drop_rate = 3.0;
                self.collection_object_hp_rate = 0.5;
                self.collection_object_respawn_speed_rate = 0.5;
                self.enemy_drop_item_rate = 2.0;
            }
            Preset::Normal => {
                // Normal preset is our base; no changes.
            }
            Preset::Hard => {
                self.day_time_speed_rate = 1.0;
                self.night_time_speed_rate = 1.0;
                self.exp_rate = 0.5;
                self.pal_capture_rate = 1.0;
                self.pal_spawn_num_rate = 1.0;
                self.pal_damage_rate_attack = 0.5;
                self.pal_damage_rate_defense = 2.0;
                self.player_damage_rate_defense = 4.0;
                self.player_stomach_decrease_rate = 1.0;
                self.player_stamina_decrease_rate = 1.0;
                self.player_damage_rate_attack = 0.7;
                self.player_auto_hp_regen_rate = 0.6;
                self.player_auto_hp_regen_rate_in_sleep = 0.6;
                self.build_object_damage_rate = 0.7;
                self.build_object_deterioration_damage_rate = 1.0;
                self.collection_drop_rate = 0.8;
                self.collection_object_hp_rate = 1.0;
                self.collection_object_respawn_speed_rate = 2.0;
                self.enemy_drop_item_rate = 0.7;
                "Drop all Items and all Pals on Team".clone_into(&mut self.death_penalty);
            }
        }
    }
}

impl Default for GameSettings {
    #[allow(clippy::too_many_lines)]
    fn default() -> Self {
        // Start with Normal preset as our base.
        let mut settings = Self::normal();

        // If a PRESET env variable is provided, override our base.
        let preset_str = fetch_var("PRESET", "");
        if !preset_str.is_empty()
            && let Ok(preset) = serde_plain::from_str::<Preset>(&preset_str)
        {
            settings.apply_preset(preset);
        }

        Self {
            difficulty: fetch_var("DIFFICULTY", &settings.difficulty),
            randomizer_type: fetch_var("RANDOMIZER_TYPE", &settings.randomizer_type),
            randomizer_seed: fetch_var("RANDOMIZER_SEED", &settings.randomizer_seed),
            is_randomizer_pal_level_random: env_parse!(
                "B_IS_RANDOMIZER_PAL_LEVEL_RANDOM",
                settings.is_randomizer_pal_level_random,
                bool
            ),
            day_time_speed_rate: env_parse!(
                "DAY_TIME_SPEED_RATE",
                settings.day_time_speed_rate,
                f32
            ),
            night_time_speed_rate: env_parse!(
                "NIGHT_TIME_SPEED_RATE",
                settings.night_time_speed_rate,
                f32
            ),
            exp_rate: env_parse!("EXP_RATE", settings.exp_rate, f32),
            pal_capture_rate: env_parse!("PAL_CAPTURE_RATE", settings.pal_capture_rate, f32),
            pal_spawn_num_rate: env_parse!("PAL_SPAWN_NUM_RATE", settings.pal_spawn_num_rate, f32),
            pal_damage_rate_attack: env_parse!(
                "PAL_DAMAGE_RATE_ATTACK",
                settings.pal_damage_rate_attack,
                f32
            ),
            pal_damage_rate_defense: env_parse!(
                "PAL_DAMAGE_RATE_DEFENSE",
                settings.pal_damage_rate_defense,
                f32
            ),
            allow_global_palbox_export: env_parse!(
                "B_ALLOW_GLOBAL_PALBOX_EXPORT",
                settings.allow_global_palbox_export,
                bool
            ),
            allow_global_palbox_import: env_parse!(
                "B_ALLOW_GLOBAL_PALBOX_IMPORT",
                settings.allow_global_palbox_import,
                bool
            ),
            character_recreate_in_hardcore: env_parse!(
                "B_CHARACTER_RECREATE_IN_HARDCORE",
                settings.character_recreate_in_hardcore,
                bool
            ),
            player_damage_rate_attack: env_parse!(
                "PLAYER_DAMAGE_RATE_ATTACK",
                settings.player_damage_rate_attack,
                f32
            ),
            player_damage_rate_defense: env_parse!(
                "PLAYER_DAMAGE_RATE_DEFENSE",
                settings.player_damage_rate_defense,
                f32
            ),
            player_stomach_decrease_rate: env_parse!(
                "PLAYER_STOMACH_DECREASE_RATE",
                settings.player_stomach_decrease_rate,
                f32
            ),
            player_stamina_decrease_rate: env_parse!(
                "PLAYER_STAMINA_DECREASE_RATE",
                settings.player_stamina_decrease_rate,
                f32
            ),
            player_auto_hp_regen_rate: env_parse!(
                "PLAYER_AUTO_HP_REGEN_RATE",
                settings.player_auto_hp_regen_rate,
                f32
            ),
            player_auto_hp_regen_rate_in_sleep: env_parse!(
                "PLAYER_AUTO_HP_REGEN_RATE_IN_SLEEP",
                settings.player_auto_hp_regen_rate_in_sleep,
                f32
            ),
            pal_stomach_decrease_rate: env_parse!(
                "PAL_STOMACH_DECREASE_RATE",
                settings.pal_stomach_decrease_rate,
                f32
            ),
            pal_stamina_decrease_rate: env_parse!(
                "PAL_STAMINA_DECREASE_RATE",
                settings.pal_stamina_decrease_rate,
                f32
            ),
            pal_auto_hp_regen_rate: env_parse!(
                "PAL_AUTO_HP_REGEN_RATE",
                settings.pal_auto_hp_regen_rate,
                f32
            ),
            pal_auto_hp_regen_rate_in_sleep: env_parse!(
                "PAL_AUTO_HP_REGEN_RATE_IN_SLEEP",
                settings.pal_auto_hp_regen_rate_in_sleep,
                f32
            ),
            build_object_hp_rate: env_parse!(
                "BUILD_OBJECT_HP_RATE",
                settings.build_object_hp_rate,
                f32
            ),
            build_object_damage_rate: env_parse!(
                "BUILD_OBJECT_DAMAGE_RATE",
                settings.build_object_damage_rate,
                f32
            ),
            build_object_deterioration_damage_rate: env_parse!(
                "BUILD_OBJECT_DETERIORATION_DAMAGE_RATE",
                settings.build_object_deterioration_damage_rate,
                f32
            ),
            collection_drop_rate: env_parse!(
                "COLLECTION_DROP_RATE",
                settings.collection_drop_rate,
                f32
            ),
            collection_object_hp_rate: env_parse!(
                "COLLECTION_OBJECT_HP_RATE",
                settings.collection_object_hp_rate,
                f32
            ),
            collection_object_respawn_speed_rate: env_parse!(
                "COLLECTION_OBJECT_RESPAWN_SPEED_RATE",
                settings.collection_object_respawn_speed_rate,
                f32
            ),
            enemy_drop_item_rate: env_parse!(
                "ENEMY_DROP_ITEM_RATE",
                settings.enemy_drop_item_rate,
                f32
            ),
            death_penalty: fetch_var("DEATH_PENALTY", &settings.death_penalty),
            enable_pvp: env_parse!("ENABLE_PVP", settings.enable_pvp, bool),
            enable_friendly_fire: env_parse!(
                "ENABLE_FRIENDLY_FIRE",
                settings.enable_friendly_fire,
                bool
            ),
            enable_invader_enemy: env_parse!(
                "ENABLE_INVADER_ENEMY",
                settings.enable_invader_enemy,
                bool
            ),
            active_unko: env_parse!("ACTIVE_UNKO", settings.active_unko, bool),
            enable_aim_assist_pad: env_parse!(
                "ENABLE_AIM_ASSIST_PAD",
                settings.enable_aim_assist_pad,
                bool
            ),
            enable_aim_assist_keyboard: env_parse!(
                "ENABLE_AIM_ASSIST_KEYBOARD",
                settings.enable_aim_assist_keyboard,
                bool
            ),
            drop_item_max_num: env_parse!("DROP_ITEM_MAX_NUM", settings.drop_item_max_num, u32),
            drop_item_max_num_unko: env_parse!(
                "DROP_ITEM_MAX_NUM_UNKO",
                settings.drop_item_max_num_unko,
                u32
            ),
            base_camp_max_num: env_parse!("BASE_CAMP_MAX_NUM", settings.base_camp_max_num, u16),
            base_camp_worker_max_num: env_parse!(
                "BASE_CAMP_WORKER_MAX_NUM",
                settings.base_camp_worker_max_num,
                u16
            ),
            drop_item_alive_max_hours: env_parse!(
                "DROP_ITEM_ALIVE_MAX_HOURS",
                settings.drop_item_alive_max_hours,
                f32
            ),
            auto_reset_guild_no_online_players: env_parse!(
                "AUTO_RESET_GUILD_NO_ONLINE_PLAYERS",
                settings.auto_reset_guild_no_online_players,
                bool
            ),
            auto_reset_guild_time_no_online_players: env_parse!(
                "AUTO_RESET_GUILD_TIME_NO_ONLINE_PLAYERS",
                settings.auto_reset_guild_time_no_online_players,
                f32
            ),
            guild_player_max_num: env_parse!(
                "GUILD_PLAYER_MAX_NUM",
                settings.guild_player_max_num,
                u16
            ),
            base_camp_max_num_in_guild: env_parse!(
                "BASE_CAMP_MAX_NUM_IN_GUILD",
                settings.base_camp_max_num_in_guild,
                u16
            ),
            pal_egg_default_hatching_time: env_parse!(
                "PAL_EGG_DEFAULT_HATCHING_TIME",
                settings.pal_egg_default_hatching_time,
                f32
            ),
            work_speed_rate: env_parse!("WORK_SPEED_RATE", settings.work_speed_rate, f32),
            auto_save_span: env_parse!("AUTO_SAVE_SPAN", settings.auto_save_span, f32),
            is_multiplay: env_parse!("IS_MULTIPLAY", settings.is_multiplay, bool),
            is_pvp: env_parse!("IS_PVP", settings.is_pvp, bool),
            hardcore: env_parse!("HARDCORE", settings.hardcore, bool),
            pal_lost: env_parse!("PAL_LOST", settings.pal_lost, bool),
            can_pickup_other_guild_death_penalty_drop: env_parse!(
                "CAN_PICKUP_OTHER_GUILD_DEATH_PENALTY_DROP",
                settings.can_pickup_other_guild_death_penalty_drop,
                bool
            ),
            enable_non_login_penalty: env_parse!(
                "ENABLE_NON_LOGIN_PENALTY",
                settings.enable_non_login_penalty,
                bool
            ),
            enable_fast_travel: env_parse!("ENABLE_FAST_TRAVEL", settings.enable_fast_travel, bool),
            is_start_location_select_by_map: env_parse!(
                "IS_START_LOCATION_SELECT_BY_MAP",
                settings.is_start_location_select_by_map,
                bool
            ),
            exist_player_after_logout: env_parse!(
                "EXIST_PLAYER_AFTER_LOGOUT",
                settings.exist_player_after_logout,
                bool
            ),
            enable_defense_other_guild_player: env_parse!(
                "ENABLE_DEFENSE_OTHER_GUILD_PLAYER",
                settings.enable_defense_other_guild_player,
                bool
            ),
            invisible_other_guild_base_camp_area_fx: env_parse!(
                "INVISIBLE_OTHER_GUILD_BASE_CAMP_AREA_FX",
                settings.invisible_other_guild_base_camp_area_fx,
                bool
            ),
            build_area_limit: env_parse!("BUILD_AREA_LIMIT", settings.build_area_limit, bool),
            item_weight_rate: env_parse!("ITEM_WEIGHT_RATE", settings.item_weight_rate, f32),
            coop_player_max_num: env_parse!(
                "COOP_PLAYER_MAX_NUM",
                settings.coop_player_max_num,
                u16
            ),
            server_player_max_num: env_parse!(
                "SERVER_PLAYER_MAX_NUM",
                settings.server_player_max_num,
                u16
            ),
            server_name: fetch_var("SERVER_NAME", &settings.server_name),
            server_description: fetch_var("SERVER_DESCRIPTION", &settings.server_description),
            admin_password: fetch_var("ADMIN_PASSWORD", &settings.admin_password),
            server_password: fetch_var("SERVER_PASSWORD", &settings.server_password),
            public_port: env_parse!("PUBLIC_PORT", settings.public_port, u16),
            public_ip: fetch_var("PUBLIC_IP", &settings.public_ip),
            rcon_enabled: env_parse!("RCON_ENABLED", settings.rcon_enabled, bool),
            rcon_port: env_parse!("RCON_PORT", settings.rcon_port, u16),
            use_auth: env_parse!("USE_AUTH", settings.use_auth, bool),
            region: fetch_var("REGION", &settings.region),
            ban_list_url: fetch_var("BAN_LIST", &settings.ban_list_url),
            restapi_enabled: env_parse!("RESTAPI_ENABLED", settings.restapi_enabled, bool),
            restapi_port: env_parse!("RESTAPI_PORT", settings.restapi_port, u16),
            show_player_list: env_parse!("SHOW_PLAYER_LIST", settings.show_player_list, bool),
            chat_post_limit_per_minute: env_parse!(
                "CHAT_POST_LIMIT_PER_MINUTE",
                settings.chat_post_limit_per_minute,
                u16
            ),
            crossplay_platforms: fetch_var("CROSSPLAY_PLATFORMS", &settings.crossplay_platforms),
            is_use_backup_save_data: env_parse!(
                "IS_USE_BACKUP_SAVE_DATA",
                settings.is_use_backup_save_data,
                bool
            ),
            log_format_type: fetch_var("LOG_FORMAT_TYPE", &settings.log_format_type),
            supply_drop_span: env_parse!("SUPPLY_DROP_SPAN", settings.supply_drop_span, f32),
            enable_predator_boss_pal: env_parse!(
                "ENABLE_PREDATOR_BOSS_PAL",
                settings.enable_predator_boss_pal,
                bool
            ),
            max_building_limit_num: env_parse!(
                "MAX_BUILDING_LIMIT_NUM",
                settings.max_building_limit_num,
                u32
            ),
            server_replicate_pawn_cull_distance: env_parse!(
                "SERVER_REPLICATE_PAWN_CULL_DISTANCE",
                settings.server_replicate_pawn_cull_distance,
                f32
            ),
        }
    }
}

/// Saves the configuration to an INI file.
///
/// Written on one line: Palworld's engine fails to parse a multi-line
/// `OptionSettings=(...)` block, so this must stay compact even though it's
/// harder to read. Use `palworld settings` to view it formatted.
pub fn save_config(path: &Path, settings: &Settings) {
    let ini_config = match to_string_compact(&settings) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("Failed to serialize config: {error}");
            return;
        }
    };

    if let Err(e) = fs::write(path, ini_config) {
        eprintln!("Failed to save config: {e}");
    }
}

/// Renders the settings that would currently be generated from the environment,
/// formatted for human reading (one entry per line) rather than the compact,
/// one-line form written to disk.
///
/// # Errors
///
/// Returns an error when `serde_json` conversion of the settings fails.
pub fn render_current_settings_pretty() -> Result<String, serde_json::Error> {
    to_string(&Settings::default())
}

/// Returns the settings that would currently be generated from the environment
/// as a structured JSON value (one field per game setting), for callers that
/// want to consume them programmatically rather than as ini text.
///
/// # Errors
///
/// Returns an error when `serde_json` conversion of the settings fails.
pub fn current_settings_json() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(GameSettings::default())
}

/// Loads the configuration from an INI file or returns defaults if the file is missing.
pub fn load_or_create_config(path: &Path) -> GameSettings {
    if let Some(parent) = path.parent()
        && !parent.exists()
        && let Err(error) = create_dir_all(parent)
    {
        eprintln!(
            "Failed to create config directory {}: {error}",
            parent.display()
        );
    }
    let default_config = Settings::default();
    save_config(path, &default_config);
    default_config.option_settings
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp, clippy::unwrap_used)]

    use super::*;
    use std::env;
    use std::fs;
    use std::path::Path;
    use std::sync::{LazyLock, Mutex};

    static TEST_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    const TEST_DIR: &str = "./tmp/tests";

    /// Helper function to reset environment variables
    fn clear_env_vars() {
        let vars = [
            "DIFFICULTY",
            "DAY_TIME_SPEED_RATE",
            "NIGHT_TIME_SPEED_RATE",
            "EXP_RATE",
            "PAL_CAPTURE_RATE",
            "PRESET",
            "SERVER_NAME",
            "ADMIN_PASSWORD",
            "SERVER_PASSWORD",
        ];
        for var in &vars {
            unsafe { env::remove_var(var) };
        }
    }

    #[test]
    fn test_default_settings() {
        let _lock = TEST_MUTEX.lock().unwrap();

        clear_env_vars();
        let settings = GameSettings::default();

        assert_eq!(settings.day_time_speed_rate, 1.0);
        assert_eq!(settings.night_time_speed_rate, 1.0);
        assert_eq!(settings.exp_rate, 1.0);
        assert_eq!(settings.pal_capture_rate, 1.0);
        assert_eq!(settings.server_name, "Default Palworld Server");
    }

    #[test]
    fn test_preset_casual() {
        let _lock = TEST_MUTEX.lock().unwrap();

        clear_env_vars();
        unsafe { env::set_var("PRESET", "casual") };

        let settings = GameSettings::default();
        assert_eq!(settings.exp_rate, 2.0);
        assert_eq!(settings.pal_capture_rate, 2.0);
        assert_eq!(settings.player_damage_rate_attack, 2.0);
        assert_eq!(settings.player_damage_rate_defense, 0.5);
        assert_eq!(settings.collection_drop_rate, 3.0);
    }

    #[test]
    fn test_preset_hard() {
        let _lock = TEST_MUTEX.lock().unwrap();

        clear_env_vars();
        unsafe { env::set_var("PRESET", "hard") };

        let settings = GameSettings::default();
        assert_eq!(settings.exp_rate, 0.5);
        assert_eq!(settings.pal_damage_rate_attack, 0.5);
        assert_eq!(settings.pal_damage_rate_defense, 2.0);
        assert_eq!(settings.player_damage_rate_attack, 0.7);
        assert_eq!(settings.enemy_drop_item_rate, 0.7);
    }

    #[test]
    fn test_env_variable_override() {
        let _lock = TEST_MUTEX.lock().unwrap();

        clear_env_vars();
        unsafe {
            env::set_var("EXP_RATE", "3.5");
            env::set_var("PAL_CAPTURE_RATE", "5.0");
        }

        let settings = GameSettings::default();
        assert_eq!(settings.exp_rate, 3.5);
        assert_eq!(settings.pal_capture_rate, 5.0);
    }

    #[test]
    fn test_env_variable_quoted_values_are_unwrapped() {
        let _lock = TEST_MUTEX.lock().unwrap();

        clear_env_vars();
        unsafe {
            // Mirrors docker-compose's `${VAR:-"default"}` syntax, which does
            // not strip the quotes the way a shell would.
            env::set_var("ADMIN_PASSWORD", "\"super-secret-password\"");
            env::set_var("EXP_RATE", "\"3.5\"");
        }

        let settings = GameSettings::default();
        assert_eq!(settings.admin_password, "super-secret-password");
        assert_eq!(settings.exp_rate, 3.5);

        clear_env_vars();
    }

    #[test]
    fn test_preset_with_env_override() {
        let _lock = TEST_MUTEX.lock().unwrap();

        clear_env_vars();
        unsafe {
            env::set_var("PRESET", "casual");
            env::set_var("EXP_RATE", "3.0"); // Override casual's 2.0 exp rate
        }

        let settings = GameSettings::default();
        assert_eq!(settings.exp_rate, 3.0);
        assert_eq!(settings.pal_capture_rate, 2.0); // Preset value remains if not overridden
    }

    #[test]
    fn test_save_and_load_config() {
        let _lock = TEST_MUTEX.lock().unwrap();

        clear_env_vars();
        let test_path = Path::new(TEST_DIR).join("test_config.ini");

        // Ensure the test directory exists
        fs::create_dir_all(TEST_DIR).unwrap();

        let settings = Settings::default();
        save_config(&test_path, &settings);

        assert!(test_path.exists());

        let loaded_settings = load_or_create_config(&test_path);
        assert_eq!(loaded_settings.server_name, "Default Palworld Server");
        assert_eq!(loaded_settings.exp_rate, 1.0);
    }

    #[test]
    fn test_save_config_writes_option_settings_on_one_line() {
        let _lock = TEST_MUTEX.lock().unwrap();

        clear_env_vars();
        let test_path = Path::new(TEST_DIR).join("test_config_compact.ini");
        fs::create_dir_all(TEST_DIR).unwrap();

        let settings = Settings::default();
        save_config(&test_path, &settings);

        let contents = fs::read_to_string(&test_path).unwrap();
        // Palworld's engine can't parse a multi-line OptionSettings block, so
        // the saved file must be exactly: header line + one OptionSettings line.
        assert_eq!(contents.lines().count(), 2);
        assert!(contents.lines().nth(1).unwrap().starts_with("OptionSettings=("));
        assert!(contents.trim_end().ends_with(')'));
    }

    /// A real `DefaultPalWorldSettings.ini` shipped by the Palworld 1.0 dedicated
    /// server (copied read-only into `resources/`; these tests only read it).
    const REFERENCE_DEFAULT_INI: &str =
        include_str!("../resources/DefaultPalWorldSettings.ini");

    /// Confirms the reference file itself keeps `OptionSettings=(...)` on one
    /// line with no newline between the opening `(` and its matching `)` —
    /// the actual engine-shipped format we're matching, not just something we
    /// assumed. If this ever fails, the fixture no longer represents what the
    /// engine writes and `save_config`'s one-line requirement should be
    /// re-checked against a fresh copy.
    #[test]
    fn reference_default_ini_keeps_option_settings_on_one_line() {
        let option_settings_lines: Vec<&str> = REFERENCE_DEFAULT_INI
            .lines()
            .filter(|line| line.starts_with("OptionSettings=("))
            .collect();

        assert_eq!(
            option_settings_lines.len(),
            1,
            "expected exactly one line starting with OptionSettings=(, meaning the whole \
             block including its closing ')' is on that same line"
        );
        assert!(option_settings_lines[0].trim_end().ends_with(')'));
    }

    /// Our own `save_config` output must have the identical shape: one line
    /// for `OptionSettings=(...)`, matching the reference file's format
    /// checked above, rather than something we only assumed the engine wants.
    #[test]
    fn save_config_output_matches_reference_one_line_shape() {
        let _lock = TEST_MUTEX.lock().unwrap();

        clear_env_vars();
        let test_path = Path::new(TEST_DIR).join("test_config_matches_reference_shape.ini");
        fs::create_dir_all(TEST_DIR).unwrap();
        save_config(&test_path, &Settings::default());
        let ours = fs::read_to_string(&test_path).unwrap();

        let reference_option_settings_lines = REFERENCE_DEFAULT_INI
            .lines()
            .filter(|line| line.starts_with("OptionSettings=("))
            .count();
        let our_option_settings_lines = ours
            .lines()
            .filter(|line| line.starts_with("OptionSettings=("))
            .count();

        assert_eq!(reference_option_settings_lines, 1);
        assert_eq!(
            our_option_settings_lines, reference_option_settings_lines,
            "our OptionSettings block must be on one line like the engine's own default ini"
        );
    }

    /// Extracts the top-level `Key=value` entries from a reference ini's
    /// `OptionSettings=(...)` body, splitting on commas at paren/quote depth 0
    /// so that values like `CrossplayPlatforms=(Steam,Xbox,PS5,Mac)` or
    /// `ServerDescription="a, b"` aren't split apart.
    fn reference_option_keys(ini: &str) -> Vec<String> {
        let body = ini
            .lines()
            .find_map(|line| line.strip_prefix("OptionSettings=("))
            .and_then(|rest| rest.strip_suffix(')'))
            .expect("fixture must contain a single-line OptionSettings=(...) block");

        let mut keys = Vec::new();
        let mut depth = 0i32;
        let mut in_quotes = false;
        let mut entry_start = 0usize;
        let chars: Vec<char> = body.chars().collect();
        for (i, &c) in chars.iter().enumerate() {
            match c {
                '"' => in_quotes = !in_quotes,
                '(' if !in_quotes => depth += 1,
                ')' if !in_quotes => depth -= 1,
                ',' if !in_quotes && depth == 0 => {
                    let entry: String = chars[entry_start..i].iter().collect();
                    if let Some(key) = entry.split('=').next() {
                        keys.push(key.to_owned());
                    }
                    entry_start = i + 1;
                }
                _ => {}
            }
        }
        let last: String = chars[entry_start..].iter().collect();
        if let Some(key) = last.split('=').next() {
            keys.push(key.to_owned());
        }
        keys
    }

    /// Every field we serialize must exist under the exact same key in the
    /// engine's own shipped defaults. This catches drift like a field being
    /// renamed to "fix" what looks like a typo, when the engine's actual ini
    /// key has kept the typo (e.g. `PlayerStomachDecreaceRate`, not
    /// `...DecreaseRate`) — a mismatch here means the value is silently
    /// dropped by the game and the engine's own default is used instead.
    #[test]
    fn all_serialized_fields_exist_in_reference_default_ini() {
        let reference_keys = reference_option_keys(REFERENCE_DEFAULT_INI);

        let settings = GameSettings::normal();
        let value = serde_json::to_value(&settings).unwrap();
        let serde_json::Value::Object(map) = value else {
            panic!("GameSettings should serialize to a JSON object");
        };

        let mut missing = Vec::new();
        for key in map.keys() {
            if !reference_keys.iter().any(|reference_key| reference_key == key) {
                missing.push(key.clone());
            }
        }

        assert!(
            missing.is_empty(),
            "these fields don't match any key in the reference DefaultPalWorldSettings.ini \
             (renamed/typo'd relative to what the engine actually expects?): {missing:?}"
        );
    }

    /// Documents (without failing) the settings Palworld 1.0 ships that our
    /// `GameSettings` doesn't model yet, so new-field coverage gaps are visible
    /// rather than silently absent.
    #[test]
    fn reference_default_ini_fields_not_yet_modeled() {
        let reference_keys = reference_option_keys(REFERENCE_DEFAULT_INI);

        let settings = GameSettings::normal();
        let value = serde_json::to_value(&settings).unwrap();
        let serde_json::Value::Object(map) = value else {
            panic!("GameSettings should serialize to a JSON object");
        };

        let unmodeled: Vec<&String> = reference_keys
            .iter()
            .filter(|key| !map.contains_key(key.as_str()))
            .collect();

        println!(
            "{} of {} reference fields are not yet modeled in GameSettings: {unmodeled:?}",
            unmodeled.len(),
            reference_keys.len()
        );
    }
}
