use std::iter::repeat_with;

use log::info;
use serde_json::{json, Value};

use anyhow::{Context, Result};

use crate::PublishTypes;

pub struct Services {
    getforplay2: &'static str,
    saveevents2: &'static str,
    contractcreatepage: &'static str,
    createfromparams: &'static str,
}

pub static HITMAN2: Services = Services {
    getforplay2:
        "https://pc2-service.hitman.io/authentication/api/userchannel/ContractsService/GetForPlay2",
    saveevents2:
        "https://pc2-service.hitman.io/authentication/api/userchannel/EventsService/SaveEvents2",
    contractcreatepage:
        "https://pc2-service.hitman.io/profiles/page/contractcreation/create",
    createfromparams:
         "https://pc2-service.hitman.io/authentication/api/userchannel/ContractsService/CreateFromParams"
};

pub static HITMAN3: Services = Services {
    getforplay2:
        "https://hm3-service.hitman.io/authentication/api/userchannel/ContractsService/GetForPlay2",
    saveevents2:
        "https://hm3-service.hitman.io/authentication/api/userchannel/EventsService/SaveEvents2",
    contractcreatepage:
        "https://hm3-service.hitman.io/profiles/page/contractcreation/create",
    createfromparams:
        "https://hm3-service.hitman.io/authentication/api/userchannel/ContractsService/CreateFromParams"
};

pub struct Contract {
    contract_json: Value,
    services: &'static Services,
    get4play: Get4PlayResponse,
    session_id: String,
    r#type:crate::PublishTypes
}

struct Get4PlayResponse {
    game_changers: Vec<String>,
    contract_session_id: String,
}

#[derive(Debug)]
enum Event {
    ContractStartEvent,
    IntroCutEndEvent,
    KillEvent { repository_id: String },
    AllBodiesHiddenEvent,
    ExitGateEvent,
}

fn get_random_session_id() -> String {
    let mut l: String = repeat_with(fastrand::alphanumeric).take(32).collect();
    l += "-";
    let r: String = repeat_with(fastrand::alphanumeric).take(10).collect();
    l += r.as_str();
    l
}

impl Contract {
    pub fn from_contract_json(
        contract: Value,
        hitman_service: crate::PublishTypes,
    ) -> Result<Self> {
        Ok(Contract {
            r#type:hitman_service,
            contract_json: contract,
            services: if let PublishTypes::HITMAN2 = hitman_service {
                &HITMAN2
            } else {
                &HITMAN3
            },
            get4play: Get4PlayResponse {
                game_changers: Vec::new(),
                contract_session_id: "".to_string(),
            },
            session_id: get_random_session_id(),
        })
    }
    pub async fn publish_contract(&mut self, user_id: &String, bearer: &String) -> Result<()> {
        self.get4play = self.get_for_play2(&bearer).await?;

        info!(
            "Contract Session ID: {}",
            &self.get4play.contract_session_id
        );

        let mut events = vec![Event::ContractStartEvent, Event::IntroCutEndEvent];

        for kill in self.contract_json["CreateFromParamsJ"]["creationData"]["Targets"]
            .as_array()
            .context("Invalid JSON")?
        {
            events.push(Event::KillEvent {
                repository_id: kill["RepositoryId"]
                    .as_str()
                    .context("Invalid JSON")?
                    .to_string(),
            });
        }

        events.push(Event::ExitGateEvent);

        self.save_events(events, user_id, bearer, &self.get4play.contract_session_id)
            .await?;

        self.create(user_id, bearer).await?;

        Ok(())
    }

    async fn create(&self, user_id: &String, bearer: &String) -> Result<()> {
        let result = reqwest::Client::new()
            .get(self.services.contractcreatepage)
            .bearer_auth(bearer)
            .header("Version", if let PublishTypes::HITMAN2=self.r#type{"7.17.0"}else{"8.7.0"})
            .send()
            .await?
            .text()
            .await?;
        info!("Contract Create Page Result:{}",result);
        let result: Value = serde_json::from_str(result.as_str())?;
        let contractid = result["data"]["Contract"]["ContractId"]
            .as_str()
            .context("Invalid Contract Page JSON")?
            .to_string();
        let contractpublicid = result["data"]["Contract"]["ContractPublicId"]
            .as_str()
            .context("Invalid Contract Page JSON")?
            .to_string();
        info!("Contract ID:{}, Public ID:{}", contractid, contractpublicid);

        let mut json = self.contract_json["CreateFromParamsJ"].clone();
        json["creationData"]["ContractId"] = serde_json::Value::String(contractid);
        json["creationData"]["ContractPublicId"] = serde_json::Value::String(contractpublicid);
        
        let result = reqwest::Client::new()
            .post(self.services.createfromparams)
            .bearer_auth(bearer)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(json.to_string())
            .send()
            .await?
            .text()
            .await?;
        info!("Finished.");
        Ok(())
    }
    async fn save_event2(
        &self,
        event: Event,
        user_id: &String,
        bearer: &String,
        contract_session_id: &String,
    ) -> Result<()> {
        let json_event = match event {
            Event::ContractStartEvent => {
                json!({
                    "Name": "ContractStart",
                    "ContractSessionId": self.get4play.contract_session_id,
                    "ContractId": self.get_contract_id()?,
                    "Value": {
                        "Loadout": [],
                        "Disguise": "00000000-0000-0000-0000-000000000000",
                        "LocationId": self.get_location_id()?,
                        "GameChangers": self.get4play.game_changers,
                        "ContractType": "creation",
                        "DifficultyLevel": 2.000000 as f32,
                        "IsHitmanSuit": true,
                        "SelectedCharacterId": "00000000-0000-0000-0000-000000000000"
                    },
                    "XboxGameMode": 2.000000 as f32,
                    "XboxDifficulty": 0.000000 as f32,
                    "Timestamp": 0.000000 as f32,
                    "UserId": user_id,
                    "SessionId": self.session_id,
                    "Origin": "gameclient",
                    "Id": uuid::Uuid::new_v4().to_string()
                })
            }
            Event::IntroCutEndEvent => {
                json!(
                        {
                            "Timestamp": 1.0 as f32,
                            "Name": "IntroCutEnd",
                            "ContractSessionId": self.get4play.contract_session_id,
                            "ContractId": self.get_contract_id()?,
                            "Value": "",
                            "UserId": user_id,
                            "SessionId": self.session_id,
                            "Origin": "gameclient",
                            "Id": uuid::Uuid::new_v4().to_string()
                        }
                )
            }
            Event::KillEvent { ref repository_id } => {
                json!(
                        {
                            "Timestamp": 1.0 as f32,
                            "Name": "Kill",
                            "ContractSessionId": self.get4play.contract_session_id,
                            "ContractId": self.get_contract_id()?,
                            "Value": {"RepositoryId":repository_id,
                            "ActorId":0.0 as f32,
                            "ActorType":1.0 as f32,
                            "KillType":1.0 as f32,
                            "KillContext":1.0 as f32,
                            "BodyPartId":1.0 as f32,
                            "RoomId":1.0 as f32,
                            "ExplosionType":1.0 as f32,
                            "TotalDamage":1000.0 as f32,
                            "Accident":true,
                            "Explosive":true,
                            "Projectile":true,
                            "Sniper":true,
                            "IsHeadshot":true,
                            "IsTarget":true,
                            "ThroughWall":true,
                            "IsMoving":true,
                            "OutfitIsHitmanSuit":true,
                            "WeaponSilenced":true,
                            "KillItemRepositoryId":"",
                            "OutfitRepositoryId":"",
                            "ActorName":"",
                            "KillClass":"",
                            "ActorPosition": "0.0, 0.0, 0.0",
                            "HeroPosition": "0.0, 0.0, 0.0",
                            "DamageEvents":[],
                            "PlayerId":0.0 as f32,
                            "KillItemInstanceId":"",
                            "KillItemCategory":"",
                            "KillMethodBroad":"",
                            "KillMethodStrict":"",
                            "History":[]},
                            "UserId": user_id,
                            "SessionId": self.session_id,
                            "Origin": "gameclient",
                            "Id": uuid::Uuid::new_v4().to_string(),

                        }
                )
            }
            Event::AllBodiesHiddenEvent => todo!(),
            Event::ExitGateEvent => {
                json!(
                        {
                            "Timestamp": 2.0 as f32,
                            "Name": "exit_gate",
                            "ContractSessionId": self.get4play.contract_session_id,
                            "ContractId": self.get_contract_id()?,
                            "Value": {
                                "name_ExitGate":"Exit_Gate",
                                "RepositoryId":self.get_exit_id()?
                            },
                            "UserId": user_id,
                            "SessionId": self.session_id,
                            "Origin": "gameclient",
                            "Id": uuid::Uuid::new_v4().to_string()
                        }
                )
            }
        };

        let json = json!({
            "userId":user_id,
            "values":[json_event]
        });

        info!("Save Events 2 Emitted. Event:{:#?}", event);
        let result = reqwest::Client::new()
            .post(self.services.saveevents2)
            .bearer_auth(bearer)
            .body(json.to_string())
            .send()
            .await?
            .text()
            .await?;

        info!("Save Events 2 Responsed. Response:${:#?}", result);

        Ok(())
    }

    async fn save_events(
        &self,
        events: Vec<Event>,
        user_id: &String,
        bearer: &String,
        contract_session_id: &String,
    ) -> Result<()> {
        let mut events_json: Vec<Value> = Vec::new();
        for event in &events {
            let json_event = match event {
                Event::ContractStartEvent => {
                    json!({
                        "Name": "ContractStart",
                        "ContractSessionId": self.get4play.contract_session_id,
                        "ContractId": self.get_contract_id()?,
                        "Value": {
                            "Loadout": [],
                            "Disguise": "00000000-0000-0000-0000-000000000000",
                            "LocationId": self.get_location_id()?,
                            "GameChangers": self.get4play.game_changers,
                            "ContractType": "creation",
                            "DifficultyLevel": 2.000000 as f32,
                            "IsHitmanSuit": true,
                            "SelectedCharacterId": "00000000-0000-0000-0000-000000000000"
                        },
                        "XboxGameMode": 2.000000 as f32,
                        "XboxDifficulty": 0.000000 as f32,
                        "Timestamp": 0.000000 as f32,
                        "UserId": user_id,
                        "SessionId": self.session_id,
                        "Origin": "gameclient",
                        "Id": uuid::Uuid::new_v4().to_string()
                    })
                }
                Event::IntroCutEndEvent => {
                    json!(
                            {
                                "Timestamp": 1.0 as f32,
                                "Name": "IntroCutEnd",
                                "ContractSessionId": self.get4play.contract_session_id,
                                "ContractId": self.get_contract_id()?,
                                "Value": "",
                                "UserId": user_id,
                                "SessionId": self.session_id,
                                "Origin": "gameclient",
                                "Id": uuid::Uuid::new_v4().to_string()
                            }
                    )
                }
                Event::KillEvent { ref repository_id } => {
                    json!(
                            {
                                "Timestamp": 1.0 as f32,
                                "Name": "Kill",
                                "ContractSessionId": self.get4play.contract_session_id,
                                "ContractId": self.get_contract_id()?,
                                "Value": {"RepositoryId":repository_id,
                                "ActorId":0.0 as f32,
                                "ActorType":1.0 as f32,
                                "KillType":1.0 as f32,
                                "KillContext":1.0 as f32,
                                "BodyPartId":1.0 as f32,
                                "RoomId":1.0 as f32,
                                "ExplosionType":1.0 as f32,
                                "TotalDamage":1000.0 as f32,
                                "Accident":true,
                                "Explosive":true,
                                "Projectile":true,
                                "Sniper":true,
                                "IsHeadshot":true,
                                "IsTarget":true,
                                "ThroughWall":true,
                                "IsMoving":true,
                                "OutfitIsHitmanSuit":true,
                                "WeaponSilenced":true,
                                "KillItemRepositoryId":"",
                                "OutfitRepositoryId":"",
                                "ActorName":"",
                                "KillClass":"",
                                "ActorPosition": "0.0, 0.0, 0.0",
                                "HeroPosition": "0.0, 0.0, 0.0",
                                "DamageEvents":[],
                                "PlayerId":0.0 as f32,
                                "KillItemInstanceId":"",
                                "KillItemCategory":"",
                                "KillMethodBroad":"",
                                "KillMethodStrict":"",
                                "History":[]},
                                "UserId": user_id,
                                "SessionId": self.session_id,
                                "Origin": "gameclient",
                                "Id": uuid::Uuid::new_v4().to_string(),

                            }
                    )
                }
                Event::AllBodiesHiddenEvent => todo!(),
                Event::ExitGateEvent => {
                    json!(
                            {
                                "Timestamp": 2.0 as f32,
                                "Name": "exit_gate",
                                "ContractSessionId": self.get4play.contract_session_id,
                                "ContractId": self.get_contract_id()?,
                                "Value": {
                                    "name_ExitGate":"Exit_Gate",
                                    "RepositoryId":self.get_exit_id()?
                                },
                                "UserId": user_id,
                                "SessionId": self.session_id,
                                "Origin": "gameclient",
                                "Id": uuid::Uuid::new_v4().to_string()
                            }
                    )
                }
            };
            events_json.push(json_event);
        }

        let json = json!({
            "userId":user_id,
            "values":events_json
        });

        info!("Save Events 2 Emitted. Event:{:#?}", events);
        let result = reqwest::Client::new()
            .post(self.services.saveevents2)
            .bearer_auth(bearer)
            .body(json.to_string())
            .send()
            .await?
            .text()
            .await?;

        info!("Save Events 2 Responsed. Response:${:#?}", result);

        Ok(())
    }

    async fn get_for_play2(&self, bearer: &String) -> Result<Get4PlayResponse> {
        let get4play2 = json!({
            "id":self.get_contract_id()?,
            "locationId":"",
            "extraGameChangerIds":[],
            "difficultyLevel":2 as i32
        });

        info!("Get4Play2 Emitted. JSON:{:#?}", get4play2);

        let result = reqwest::Client::new()
            .post(self.services.getforplay2)
            .bearer_auth(bearer)
            .body(get4play2.to_string())
            .send()
            .await?
            .text()
            .await?;

        info!("Get4Play2 Response JSON:{:#?}", result);

        let result: Value = serde_json::from_str(result.as_str())?;

        Ok(Get4PlayResponse {
            contract_session_id: String::from(
                result["ContractSessionId"]
                    .as_str()
                    .context("Wrong JSON:Contract Session ID not found.")?,
            ),
            game_changers: serde_json::from_value(
                result["Contract"]["Data"]["GameChangers"].clone(),
            )?,
        })
    }
    fn get_contract_id(&self) -> Result<String> {
        Ok(self.contract_json["MissionId"]
            .as_str()
            .context("Invalid contract json: No Mission Id Found")?
            .to_string())
    }
    fn get_exit_id(&self) -> Result<String> {
        Ok(self.contract_json["ExitId"]
            .as_str()
            .context("Invalid contract json: No Mission Id Found")?
            .to_string())
    }
    fn get_location_id(&self) -> Result<String> {
        Ok(self.contract_json["MissionName"]
            .as_str()
            .context("Invalid contract json: No MissionName Found")?
            .to_string())
    }
}
