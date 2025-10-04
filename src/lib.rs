// ============================================
// SUITE COMPLETA DE TESTES - FACTORY MACROS
// Com DatabaseConnection direto
// ============================================

// Cargo.toml dependencies:
// [dependencies]
// sea-orm = { version = "1.1", features = ["sqlx-sqlite", "runtime-tokio-native-tls", "macros"] }
// tokio = { version = "1", features = ["full"] }
// uuid = { version = "1", features = ["v4"] }
// chrono = "0.4"
// paste = "1.0"
//
// [dev-dependencies]
// tokio = { version = "1", features = ["full", "test-util"] }

// ============================================
// MACRO DEFINITION
// ============================================

#[macro_export]
macro_rules! define_factory {
    (
        $(#[$meta:meta])*
        $fn_name:ident => $model:path {
            active_model: $active_model:path,
            fields: {
                $($field:ident: $field_type:ty = $default:expr),* $(,)?
            }
            $(, lazy_fields: {
                $($lazy_field:ident: $lazy_type:ty = $lazy_default:expr),* $(,)?
            })?
        }
    ) => {
        ::paste::paste! {
            // Função factory principal
            $(#[$meta])*
            pub async fn [<create_ $fn_name>](db: &sea_orm::DatabaseConnection) -> Result<$model, sea_orm::DbErr> {
                type Active = $active_model;
                let model = Active {
                    $(
                        $field: sea_orm::ActiveValue::Set($default),
                    )*
                    $($(
                        $lazy_field: sea_orm::ActiveValue::Set($lazy_default(db).await?),
                    )*)?
                    ..Default::default()
                };
                model.insert(db).await
            }

            // Builder struct
            #[derive(Debug, Clone)]
            #[allow(dead_code)]
            pub struct [<Create $fn_name:camel Builder>] {
                $(
                    $field: $field_type,
                )*
                $($(
                    $lazy_field: Option<$lazy_type>,
                )*)?
            }

            impl Default for [<Create $fn_name:camel Builder>] {
                fn default() -> Self {
                    Self::new()
                }
            }

            impl [<Create $fn_name:camel Builder>] {
                /// Cria um novo builder com valores padrão
                pub fn new() -> Self {
                    Self {
                        $(
                            $field: $default,
                        )*
                        $($(
                            $lazy_field: None,
                        )*)?
                    }
                }

                $(
                    /// Define o valor de $field
                    #[allow(dead_code)]
                    pub fn $field(mut self, value: $field_type) -> Self {
                        self.$field = value;
                        self
                    }
                )*

                $($(
                    /// Define o valor de $lazy_field
                    pub fn $lazy_field(mut self, value: $lazy_type) -> Self {
                        self.$lazy_field = Some(value);
                        self
                    }
                )*)?

                /// Constrói e salva o model no banco
                pub async fn create(self, db: &sea_orm::DatabaseConnection) -> Result<$model, sea_orm::DbErr> {
                    type Active = $active_model;
                    Active {
                        $(
                            $field: sea_orm::ActiveValue::Set(self.$field),
                        )*
                        $($(
                            $lazy_field: sea_orm::ActiveValue::Set(
                                match self.$lazy_field {
                                    Some(val) => val,
                                    None => $lazy_default(db).await?,
                                }
                            ),
                        )*)?
                        ..Default::default()
                    }
                    .insert(db)
                    .await
                }

                /// Constrói o model sem salvar (requer todos os valores lazy)
                #[allow(dead_code)]
                pub fn build(self) -> $active_model {
                    type Active = $active_model;
                    Active {
                        $(
                            $field: sea_orm::ActiveValue::Set(self.$field),
                        )*
                        $($(
                            $lazy_field: sea_orm::ActiveValue::Set(
                                self.$lazy_field.expect(concat!("lazy field '", stringify!($lazy_field), "' must be set when using build()"))
                            ),
                        )*)?
                        ..Default::default()
                    }
                }
            }

            /// Helper function para criar o builder
            #[allow(dead_code)]
            pub fn [<create_ $fn_name _builder>]() -> [<Create $fn_name:camel Builder>] {
                [<Create $fn_name:camel Builder>]::new()
            }

            /// Helper function to get model id
            #[allow(dead_code)]
            pub async fn [<get_ $fn_name _id>](db: &sea_orm::DatabaseConnection) -> Result<i32, sea_orm::DbErr> {
                [<create_ $fn_name>](db).await.map(|s| s.id)
            }
        }
    };
}

// ============================================
// DEFININDO AS FACTORIES
// ============================================

#[cfg(test)]
mod factory_tests {
    use super::*;
    use chrono::Datelike;
    use sea_orm::{
        ActiveModelTrait, ActiveValue, Database, DatabaseConnection, Schema, entity::prelude::*,
    };
    use uuid::Uuid;

    pub mod specialties {
        use super::*;

        #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
        #[sea_orm(table_name = "specialties")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            pub uuid: Uuid,
            pub name: String,
            pub description: Option<String>,
            pub is_active: bool,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }

    pub mod doctors {
        use super::*;

        #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
        #[sea_orm(table_name = "doctors")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            pub uuid: Uuid,
            pub first_name: String,
            pub last_name: String,
            pub email: String,
            pub specialty_id: i32,
            pub license_number: String,
            pub phone: Option<String>,
            pub is_active: bool,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {
            #[sea_orm(
                belongs_to = "super::specialties::Entity",
                from = "Column::SpecialtyId",
                to = "super::specialties::Column::Id"
            )]
            Specialty,
        }

        impl Related<super::specialties::Entity> for Entity {
            fn to() -> RelationDef {
                Relation::Specialty.def()
            }
        }

        impl ActiveModelBehavior for ActiveModel {}
    }

    pub mod patients {
        use super::*;

        #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
        #[sea_orm(table_name = "patients")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            pub uuid: Uuid,
            pub first_name: String,
            pub last_name: String,
            pub email: String,
            pub date_of_birth: chrono::NaiveDate,
            pub phone: String,
            pub address: Option<String>,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }

    /// Setup de banco em memória SQLite
    async fn setup_test_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("Failed to connect to test database");

        let schema = Schema::new(sea_orm::DatabaseBackend::Sqlite);

        let stmt = schema.create_table_from_entity(specialties::Entity);
        db.execute(db.get_database_backend().build(&stmt))
            .await
            .expect("Failed to create specialties table");

        let stmt = schema.create_table_from_entity(doctors::Entity);
        db.execute(db.get_database_backend().build(&stmt))
            .await
            .expect("Failed to create doctors table");

        let stmt = schema.create_table_from_entity(patients::Entity);
        db.execute(db.get_database_backend().build(&stmt))
            .await
            .expect("Failed to create patients table");

        db
    }

    async fn count_specialties(db: &DatabaseConnection) -> Result<u64, DbErr> {
        use sea_orm::EntityTrait;
        specialties::Entity::find().count(db).await
    }

    async fn count_doctors(db: &DatabaseConnection) -> Result<u64, DbErr> {
        use sea_orm::EntityTrait;
        doctors::Entity::find().count(db).await
    }

    async fn count_patients(db: &DatabaseConnection) -> Result<u64, DbErr> {
        use sea_orm::EntityTrait;
        patients::Entity::find().count(db).await
    }

    async fn find_specialty_by_id(
        db: &DatabaseConnection,
        id: i32,
    ) -> Result<Option<specialties::Model>, DbErr> {
        use sea_orm::EntityTrait;
        specialties::Entity::find_by_id(id).one(db).await
    }

    define_factory! {
        /// Cria uma specialty de teste
        specialty => specialties::Model {
            active_model: specialties::ActiveModel,
            fields: {
                name: String = "Test Specialty".to_string(),
                description: Option<String> = Some("Test Description".to_string()),
                uuid: Uuid = Uuid::new_v4(),
                is_active: bool = true,
            }
        }
    }

    // async fn get_specialty_id(db: &DatabaseConnection) -> Result<i32, DbErr> {
    //    create_specialty(db).await.map(|s| s.id)
    // }

    define_factory! {
        /// Cria um doctor de teste
        doctor => doctors::Model {
            active_model: doctors::ActiveModel,
            fields: {
                first_name: String = "John".to_string(),
                last_name: String = "Doe".to_string(),
                email: String = format!("doctor_{}@example.com", Uuid::new_v4()),
                license_number: String = format!("LIC{}", Uuid::new_v4().to_string()[..8].to_uppercase()),
                uuid: Uuid = Uuid::new_v4(),
                phone: Option<String> = Some("+5511999999999".to_string()),
                is_active: bool = true,
            },
            lazy_fields: {
                specialty_id: i32 = get_specialty_id,
            }
        }
    }

    define_factory! {
        /// Cria um patient de teste
        patient => patients::Model {
            active_model: patients::ActiveModel,
            fields: {
                first_name: String = "Maria".to_string(),
                last_name: String = "Silva".to_string(),
                email: String = format!("patient_{}@example.com", Uuid::new_v4()),
                date_of_birth: chrono::NaiveDate = chrono::NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
                phone: String = "+5511988888888".to_string(),
                uuid: Uuid = Uuid::new_v4(),
                address: Option<String> = Some("Rua Teste, 123".to_string()),
            }
        }
    }

    mod specialties_factory_tests {
        use super::*;

        #[tokio::test]
        async fn test_create_specialty_with_defaults() {
            let db = setup_test_db().await;
            let specialty = create_specialty(&db).await.unwrap();

            assert_eq!(specialty.name, "Test Specialty");
            assert_eq!(specialty.description, Some("Test Description".to_string()));
            assert!(specialty.id > 0);
            assert!(specialty.is_active);
        }

        #[tokio::test]
        async fn test_create_multiple_specialties() {
            let db = setup_test_db().await;

            let specialty1 = create_specialty(&db).await.unwrap();
            let specialty2 = create_specialty(&db).await.unwrap();
            let specialty3 = create_specialty(&db).await.unwrap();

            assert_ne!(specialty1.id, specialty2.id);
            assert_ne!(specialty2.id, specialty3.id);
            assert_ne!(specialty1.uuid, specialty2.uuid);

            let count = count_specialties(&db).await.unwrap();
            assert_eq!(count, 3);
        }

        #[tokio::test]
        async fn test_specialty_uuid_is_unique() {
            let db = setup_test_db().await;

            let specialty1 = create_specialty(&db).await.unwrap();
            let specialty2 = create_specialty(&db).await.unwrap();

            assert_ne!(specialty1.uuid, specialty2.uuid);
        }

        #[tokio::test]
        async fn test_auto_increment_id() {
            let db = setup_test_db().await;

            let specialty1 = create_specialty(&db).await.unwrap();
            let specialty2 = create_specialty(&db).await.unwrap();
            let specialty3 = create_specialty(&db).await.unwrap();

            assert_eq!(specialty1.id + 1, specialty2.id);
            assert_eq!(specialty2.id + 1, specialty3.id);
        }

        #[tokio::test]
        async fn test_specialty_persisted_correctly() {
            let db = setup_test_db().await;

            let created = create_specialty(&db).await.unwrap();
            let found = find_specialty_by_id(&db, created.id).await.unwrap();

            assert!(found.is_some());
            let found = found.unwrap();
            assert_eq!(found.id, created.id);
            assert_eq!(found.name, created.name);
            assert_eq!(found.uuid, created.uuid);
        }
    }

    // ============================================
    // TESTES - BUILDER PATTERN
    // ============================================

    mod builder_factory_tests {
        use super::*;
        #[tokio::test]
        async fn test_builder_with_default_values() {
            let db = setup_test_db().await;
            let specialty = CreateSpecialtyBuilder::new().create(&db).await.unwrap();

            assert_eq!(specialty.name, "Test Specialty");
            assert_eq!(specialty.description, Some("Test Description".to_string()));
        }

        #[tokio::test]
        async fn test_builder_with_custom_name() {
            let db = setup_test_db().await;

            let specialty = CreateSpecialtyBuilder::new()
                .name("Cardiology".to_string())
                .create(&db)
                .await
                .unwrap();

            assert_eq!(specialty.name, "Cardiology");
        }

        #[tokio::test]
        async fn test_builder_with_multiple_overrides() {
            let db = setup_test_db().await;

            let custom_uuid = Uuid::new_v4();
            let specialty = CreateSpecialtyBuilder::new()
                .name("Neurology".to_string())
                .description(Some("Brain specialist".to_string()))
                .uuid(custom_uuid)
                .is_active(false)
                .create(&db)
                .await
                .unwrap();

            assert_eq!(specialty.name, "Neurology");
            assert_eq!(specialty.description, Some("Brain specialist".to_string()));
            assert_eq!(specialty.uuid, custom_uuid);
            assert!(!specialty.is_active);
        }

        #[tokio::test]
        async fn test_builder_with_none_description() {
            let db = setup_test_db().await;

            let specialty = CreateSpecialtyBuilder::new()
                .name("Surgery".to_string())
                .description(None)
                .create(&db)
                .await
                .unwrap();

            assert_eq!(specialty.name, "Surgery");
            assert_eq!(specialty.description, None);
        }

        #[tokio::test]
        async fn test_builder_helper_function() {
            let db = setup_test_db().await;

            let specialty = create_specialty_builder()
                .name("Oncology".to_string())
                .create(&db)
                .await
                .unwrap();

            assert_eq!(specialty.name, "Oncology");
        }

        #[tokio::test]
        async fn test_builder_reuse() {
            let db = setup_test_db().await;

            let base_builder = CreateSpecialtyBuilder::new().name("Base Specialty".to_string());

            let specialty1 = base_builder.clone().create(&db).await.unwrap();
            let specialty2 = base_builder.create(&db).await.unwrap();

            assert_eq!(specialty1.name, "Base Specialty");
            assert_eq!(specialty2.name, "Base Specialty");
            assert_ne!(specialty1.id, specialty2.id);
        }

        #[test]
        fn test_build_returns_active_model() {
            let active_model = CreateSpecialtyBuilder::new()
                .name("Test".to_string())
                .build();

            assert!(matches!(active_model.name, ActiveValue::Set(_)));
            if let ActiveValue::Set(name) = active_model.name {
                assert_eq!(name, "Test");
            }
        }

        #[test]
        fn test_build_does_not_require_database() {
            let active_model = CreateSpecialtyBuilder::new()
                .name("No DB Test".to_string())
                .description(Some("Works without database".to_string()))
                .build();

            if let ActiveValue::Set(name) = active_model.name {
                assert_eq!(name, "No DB Test");
            }
        }

        #[test]
        fn test_build_with_custom_uuid() {
            let custom_uuid = Uuid::new_v4();
            let active_model = CreateSpecialtyBuilder::new().uuid(custom_uuid).build();

            if let ActiveValue::Set(uuid) = active_model.uuid {
                assert_eq!(uuid, custom_uuid);
            }
        }

        #[test]
        fn test_build_all_fields_set() {
            let active_model = CreateSpecialtyBuilder::new()
                .name("Complete".to_string())
                .description(Some("Full description".to_string()))
                .uuid(Uuid::new_v4())
                .is_active(false)
                .build();

            assert!(matches!(active_model.name, ActiveValue::Set(_)));
            assert!(matches!(active_model.description, ActiveValue::Set(_)));
            assert!(matches!(active_model.uuid, ActiveValue::Set(_)));
            assert!(matches!(active_model.is_active, ActiveValue::Set(_)));
        }
    }

    mod patient_factory_tests {
        use super::*;

        #[tokio::test]
        async fn test_create_patient_with_defaults() {
            let db = setup_test_db().await;
            let patient = create_patient(&db).await.unwrap();

            assert_eq!(patient.first_name, "Maria");
            assert_eq!(patient.last_name, "Silva");
            assert!(patient.email.contains("@example.com"));
            assert_eq!(patient.phone, "+5511988888888");
        }

        #[tokio::test]
        async fn test_create_patient_with_custom_data() {
            let db = setup_test_db().await;

            let patient = CreatePatientBuilder::new()
                .first_name("Carlos".to_string())
                .last_name("Santos".to_string())
                .email("carlos@example.com".to_string())
                .date_of_birth(chrono::NaiveDate::from_ymd_opt(1985, 5, 15).unwrap())
                .phone("+5511977777777".to_string())
                .address(Some("Av. Paulista, 1000".to_string()))
                .create(&db)
                .await
                .unwrap();

            assert_eq!(patient.first_name, "Carlos");
            assert_eq!(patient.date_of_birth.year(), 1985);
            assert_eq!(patient.address, Some("Av. Paulista, 1000".to_string()));
        }

        #[tokio::test]
        async fn test_create_multiple_patients() {
            let db = setup_test_db().await;

            for i in 0..5 {
                let _ = CreatePatientBuilder::new()
                    .first_name(format!("Patient{}", i))
                    .create(&db)
                    .await
                    .unwrap();
            }

            let count = count_patients(&db).await.unwrap();
            assert_eq!(count, 5);
        }
    }

    mod integration_tests {
        use super::*;

        #[tokio::test]
        async fn test_full_hospital_scenario() {
            let db = setup_test_db().await;

            let cardiology = create_specialty_builder()
                .name("Cardiology".to_string())
                .create(&db)
                .await
                .unwrap();

            let neurology = create_specialty_builder()
                .name("Neurology".to_string())
                .create(&db)
                .await
                .unwrap();

            let _doctor1 = create_doctor_builder()
                .specialty_id(cardiology.id)
                .first_name("Dr. House".to_string())
                .create(&db)
                .await
                .unwrap();

            let _doctor2 = create_doctor_builder()
                .specialty_id(neurology.id)
                .first_name("Dr. Strange".to_string())
                .create(&db)
                .await
                .unwrap();

            let _patient1 = create_patient_builder()
                .first_name("John".to_string())
                .create(&db)
                .await
                .unwrap();

            let _patient2 = create_patient_builder()
                .first_name("Jane".to_string())
                .create(&db)
                .await
                .unwrap();

            assert_eq!(count_specialties(&db).await.unwrap(), 2);
            assert_eq!(count_doctors(&db).await.unwrap(), 2);
            assert_eq!(count_patients(&db).await.unwrap(), 2);
        }
    }

    // ============================================
    // TESTES - EDGE CASES
    // ============================================

    mod edge_case_tests {
        use super::*;

        #[tokio::test]
        async fn test_special_characters() {
            let db = setup_test_db().await;

            let specialty = CreateSpecialtyBuilder::new()
                .name("Test 特殊 🏥".to_string())
                .description(Some("With émojis".to_string()))
                .create(&db)
                .await
                .unwrap();

            assert_eq!(specialty.name, "Test 特殊 🏥");
        }

        #[tokio::test]
        async fn test_uuid_uniqueness() {
            let db = setup_test_db().await;
            let mut uuids = std::collections::HashSet::new();

            for _ in 0..50 {
                let specialty = create_specialty(&db).await.unwrap();
                assert!(uuids.insert(specialty.uuid));
            }

            assert_eq!(uuids.len(), 50);
        }
    }
}
