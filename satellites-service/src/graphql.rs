use std::str::FromStr;

use async_graphql::*;
use async_graphql::guard::Guard;
use chrono::prelude::NaiveDate;
use strum_macros::EnumString;

use crate::{get_conn_from_ctx, Role};
use crate::persistence::model::SatelliteEntity;
use crate::persistence::repository;

pub type AppSchema = Schema<Query, EmptyMutation, EmptySubscription>;

pub struct Query;

#[Object(extends)]
impl Query {
    async fn satellites(&self, ctx: &Context<'_>) -> Vec<Satellite> {
        let satellite_entities = repository::all(&get_conn_from_ctx(ctx)).expect("Can't get satellites");

        satellite_entities.iter()
            .map(|e| { Satellite::from(e) })
            .collect()
    }

    async fn satellite(&self, ctx: &Context<'_>, id: ID) -> Option<Satellite> {
        let id = id.to_string().parse::<i32>().expect("Can't get id from String");
        repository::get(id, &get_conn_from_ctx(ctx)).ok()
            .map(|e| { Satellite::from(&e) })
    }

    #[entity]
    async fn get_planet_by_id(&self, id: ID) -> Planet {
        Planet { id: id.clone() }
    }
}

#[SimpleObject]
struct Satellite {
    id: ID,
    name: String,
    #[field(guard(RoleGuard(role = "Role::Admin")))]
    life_exists: LifeExists,
    first_spacecraft_landing_date: Option<NaiveDate>,
}

#[Enum]
#[derive(EnumString)]
enum LifeExists {
    Yes,
    OpenQuestion,
    NoData,
}

#[derive(Clone)]
struct Planet {
    id: ID
}

#[Object(extends)]
impl Planet {
    #[field(external)]
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn satellites(&self, ctx: &Context<'_>) -> Vec<Satellite> {
        let id = self.id.to_string().parse::<i32>().expect("Can't get id from String");
        let satellite_entities = repository::get_by_planet_id(id, &get_conn_from_ctx(ctx)).expect("Can't get satellites of planet");

        satellite_entities.iter()
            .map(|e| { Satellite::from(e) })
            .collect()
    }
}

impl From<&SatelliteEntity> for Satellite {
    fn from(entity: &SatelliteEntity) -> Self {
        Satellite {
            id: entity.id.into(),
            name: entity.name.clone(),
            life_exists: LifeExists::from_str(entity.life_exists.as_str()).expect("Can't convert &str to LifeExists"),
            first_spacecraft_landing_date: entity.first_spacecraft_landing_date,
        }
    }
}

struct RoleGuard {
    role: Role,
}

#[async_trait::async_trait]
impl Guard for RoleGuard {
    async fn check(&self, ctx: &Context<'_>) -> FieldResult<()> {
        if ctx.data_opt::<Role>() == Some(&self.role) {
            Ok(())
        } else {
            Err("Forbidden".into())
        }
    }
}
