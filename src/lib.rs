// ============================================
// MACRO PARA CRIAR FUNÇÕES FACTORY
// ============================================

use sea_orm::{ActiveModelTrait, ActiveValue, DbErr};
use uuid::Uuid;

// Contexto da aplicação (adapte ao seu caso)
pub struct AppContext {
    pub db: sea_orm::DatabaseConnection,
}

// ============================================
// VERSÃO 1: Macro Básica - Campos Fixos
// ============================================

/// Cria uma função factory com valores padrão fixos
macro_rules! factory {
    (
        // Nome da função a ser criada
        fn $fn_name:ident (ctx: &AppContext) -> $model:ty {
            // Campos com valores padrão
            $($field:ident: $default:expr),* $(,)?
        }
    ) => {
        pub async fn $fn_name(ctx: &AppContext) -> Result<$model, DbErr> {
            let model = <$model>::ActiveModel {
                $(
                    $field: ActiveValue::Set($default),
                )*
                ..Default::default()
            };
            model.insert(&ctx.db).await
        }
    };
}

// Exemplo de uso:
// factory! {
//     fn create_specialty(ctx: &AppContext) -> specialties::Model {
//         name: "Test Specialty".to_string(),
//         description: Some("Test Description".to_string()),
//         uuid: Uuid::new_v4(),
//     }
// }

// ============================================
// VERSÃO 2: Macro Avançada - Com Customização
// ============================================

/// Cria uma função factory que aceita overrides opcionais
macro_rules! factory_with_overrides {
    (
        fn $fn_name:ident(ctx: &AppContext) -> $model:ty {
            defaults: {
                $($field:ident: $default:expr),* $(,)?
            }
        }
    ) => {
        // Cria um struct para os overrides
        paste::paste! {
            #[derive(Default)]
            pub struct [<$fn_name:camel Overrides>] {
                $(
                    pub $field: Option<<$model as FactoryModel>::[<$field:camel Type>]>,
                )*
            }
        }

        // Cria a função factory
        pub async fn $fn_name(
            ctx: &AppContext,
            overrides: Option<paste::paste! { [<$fn_name:camel Overrides>] }>,
        ) -> Result<$model, DbErr> {
            let overrides = overrides.unwrap_or_default();
            
            let model = <$model>::ActiveModel {
                $(
                    $field: ActiveValue::Set(
                        overrides.$field.unwrap_or_else(|| $default)
                    ),
                )*
                ..Default::default()
            };
            
            model.insert(&ctx.db).await
        }
    };
}

// ============================================
// VERSÃO 3: Macro Completa - Builder Pattern
// ============================================

/// Cria uma função factory + struct builder
macro_rules! create_factory {
    (
        $fn_name:ident => $model:ty {
            $($field:ident: $default:expr),* $(,)?
        }
    ) => {
        paste::paste! {
            // Struct para configurar a factory
            #[derive(Clone)]
            pub struct [<$fn_name:camel Builder>] {
                $(
                    $field: [<$field:camel ValueType>],
                )*
            }

            impl [<$fn_name:camel Builder>] {
                pub fn new() -> Self {
                    Self {
                        $(
                            $field: $default,
                        )*
                    }
                }

                $(
                    pub fn $field(mut self, value: [<$field:camel ValueType>]) -> Self {
                        self.$field = value;
                        self
                    }
                )*

                pub async fn create(self, ctx: &AppContext) -> Result<$model, DbErr> {
                    let model = <$model>::ActiveModel {
                        $(
                            $field: ActiveValue::Set(self.$field),
                        )*
                        ..Default::default()
                    };
                    model.insert(&ctx.db).await
                }
            }

            // Função de conveniência
            pub async fn $fn_name(ctx: &AppContext) -> Result<$model, DbErr> {
                [<$fn_name:camel Builder>]::new().create(ctx).await
            }
        }
    };
}

// ============================================
// VERSÃO 4: Macro Final - Mais Prática e Flexível
// ============================================

/// Macro principal para criar factories
/// Suporta valores padrão e tipos explícitos
macro_rules! factory_fn {
    (
        // Assinatura da função
        $fn_name:ident($ctx:ident: &AppContext) -> $model_path:path {
            // Model type
            model: $active_model:ty,
            // Campos com valores padrão
            defaults: {
                $($field:ident: $field_type:ty = $default:expr),* $(,)?
            }
            $(,)?
        }
    ) => {
        pub async fn $fn_name($ctx: &AppContext) -> Result<$model_path, DbErr> {
            let model = <$active_model> {
                $(
                    $field: ActiveValue::Set($default),
                )*
                ..Default::default()
            };
            model.insert(&$ctx.db).await
        }

        // Também cria uma versão com builder
        paste::paste! {
            pub struct [<$fn_name:camel Builder>] {
                $(
                    $field: $field_type,
                )*
            }

            impl [<$fn_name:camel Builder>] {
                pub fn new() -> Self {
                    Self {
                        $(
                            $field: $default,
                        )*
                    }
                }

                $(
                    pub fn $field(mut self, value: $field_type) -> Self {
                        self.$field = value;
                        self
                    }
                )*

                pub async fn create(self, ctx: &AppContext) -> Result<$model_path, DbErr> {
                    let model = <$active_model> {
                        $(
                            $field: ActiveValue::Set(self.$field),
                        )*
                        ..Default::default()
                    };
                    model.insert(&ctx.db).await
                }
            }
        }
    };
}

// ============================================
// VERSÃO 5: Macro Mais Simples e Prática
// ============================================

/// A versão mais simples e direta
macro_rules! def_factory {
    (
        $fn_name:ident => $model:path, $active_model:path {
            $($field:ident: $default:expr),* $(,)?
        }
    ) => {
        pub async fn $fn_name(ctx: &AppContext) -> Result<$model, DbErr> {
            let model = $active_model {
                $(
                    $field: ActiveValue::Set($default),
                )*
                ..Default::default()
            };
            model.insert(&ctx.db).await
        }
    };
}

// ============================================
// EXEMPLOS DE USO COM SPECIALTIES
// ============================================

// Assumindo que você tem este módulo:
pub mod specialties {
    use sea_orm::entity::prelude::*;
    
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "specialties")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub uuid: Uuid,
        pub name: String,
        pub description: Option<String>,
    }
    
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    
    impl ActiveModelBehavior for ActiveModel {}
}

// ============================================
// USO 1: Versão Simples
// ============================================

def_factory! {
    create_specialty => specialties::Model, specialties::ActiveModel {
        name: "Test Specialty".to_string(),
        description: Some("Test Description".to_string()),
        uuid: Uuid::new_v4(),
    }
}

// Usar assim:
// let specialty = create_specialty(&ctx).await?;

// ============================================
// USO 2: Com Builder Pattern
// ============================================

factory_fn! {
    create_specialty_v2(ctx: &AppContext) -> specialties::Model {
        model: specialties::ActiveModel,
        defaults: {
            name: String = "Test Specialty".to_string(),
            description: Option<String> = Some("Test Description".to_string()),
            uuid: Uuid = Uuid::new_v4(),
        }
    }
}

// Usar assim:
// let specialty = create_specialty_v2(&ctx).await?;
// 
// Ou com customização:
// let specialty = CreateSpecialtyV2Builder::new()
//     .name("Custom Specialty".to_string())
//     .create(&ctx)
//     .await?;

// ============================================
// VERSÃO FINAL: Macro Mais Completa
// ============================================

/// Macro que cria factory function + builder + helper
macro_rules! define_factory {
    (
        $(#[$meta:meta])*
        $fn_name:ident => $model:path {
            active_model: $active_model:path,
            fields: {
                $($field:ident: $field_type:ty = $default:expr),* $(,)?
            }
            $(, context: $ctx_field:ident)?
        }
    ) => {
        // Função factory principal
        $(#[$meta])*
        pub async fn $fn_name(ctx: &AppContext) -> Result<$model, DbErr> {
            let model = $active_model {
                $(
                    $field: ActiveValue::Set($default),
                )*
                ..Default::default()
            };
            model.insert(&ctx.db).await
        }

        // Builder struct
        paste::paste! {
            #[derive(Debug, Clone)]
            pub struct [<$fn_name:camel Builder>] {
                $(
                    $field: $field_type,
                )*
            }

            impl Default for [<$fn_name:camel Builder>] {
                fn default() -> Self {
                    Self::new()
                }
            }

            impl [<$fn_name:camel Builder>] {
                /// Cria um novo builder com valores padrão
                pub fn new() -> Self {
                    Self {
                        $(
                            $field: $default,
                        )*
                    }
                }

                $(
                    /// Define o valor de $field
                    pub fn $field(mut self, value: $field_type) -> Self {
                        self.$field = value;
                        self
                    }
                )*

                /// Constrói e salva o model no banco
                pub async fn create(self, ctx: &AppContext) -> Result<$model, DbErr> {
                    let model = $active_model {
                        $(
                            $field: ActiveValue::Set(self.$field),
                        )*
                        ..Default::default()
                    };
                    model.insert(&ctx.db).await
                }

                /// Constrói o model sem salvar (útil para testes)
                pub fn build(self) -> $active_model {
                    $active_model {
                        $(
                            $field: ActiveValue::Set(self.$field),
                        )*
                        ..Default::default()
                    }
                }
            }

            /// Helper function para criar o builder
            pub fn [<$fn_name _builder>]() -> [<$fn_name:camel Builder>] {
                [<$fn_name:camel Builder>]::new()
            }
        }
    };
}

// ============================================
// EXEMPLO COMPLETO DE USO
// ============================================

define_factory! {
    /// Cria uma specialty de teste
    create_specialty_final => specialties::Model {
        active_model: specialties::ActiveModel,
        fields: {
            name: String = "Test Specialty".to_string(),
            description: Option<String> = Some("Test Description".to_string()),
            uuid: Uuid = Uuid::new_v4(),
        }
    }
}

// ============================================
// COMO USAR NOS TESTES
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_factory_usage() {
        let ctx = setup_test_context().await;

        // Uso 1: Valores padrão
        let specialty1 = create_specialty_final(&ctx).await.unwrap();
        assert_eq!(specialty1.name, "Test Specialty");

        // Uso 2: Com builder e valores customizados
        let specialty2 = create_specialty_final_builder()
            .name("Cardiology".to_string())
            .description(Some("Heart specialist".to_string()))
            .create(&ctx)
            .await
            .unwrap();
        assert_eq!(specialty2.name, "Cardiology");

        // Uso 3: Builder com chain
        let specialty3 = CreateSpecialtyFinalBuilder::new()
            .name("Neurology".to_string())
            .create(&ctx)
            .await
            .unwrap();

        // Uso 4: Build sem salvar (para testes que não precisam de DB)
        let active_model = create_specialty_final_builder()
            .name("Test".to_string())
            .build();
    }

    async fn setup_test_context() -> AppContext {
        // Setup do contexto de teste
        todo!("Implementar setup")
    }
}

// ============================================
// BÔNUS: Macro para múltiplas factories
// ============================================

macro_rules! factories {
    (
        $(
            $fn_name:ident => $model:path {
                active_model: $active_model:path,
                fields: { $($field:ident: $field_type:ty = $default:expr),* $(,)? }
            }
        )*
    ) => {
        $(
            define_factory! {
                $fn_name => $model {
                    active_model: $active_model,
                    fields: { $($field: $field_type = $default),* }
                }
            }
        )*
    };
}

// Criar várias factories de uma vez:
/*
factories! {
    create_specialty => specialties::Model {
        active_model: specialties::ActiveModel,
        fields: {
            name: String = "Test".to_string(),
            uuid: Uuid = Uuid::new_v4(),
        }
    }

    create_doctor => doctors::Model {
        active_model: doctors::ActiveModel,
        fields: {
            first_name: String = "John".to_string(),
            last_name: String = "Doe".to_string(),
            uuid: Uuid = Uuid::new_v4(),
        }
    }
}
*/

// ============================================
// RESUMO
// ============================================

/*
TODAS AS VERSÕES CRIADAS:

1. factory! - Básica, valores fixos
2. factory_with_overrides! - Com opção de override
3. create_factory! - Com builder pattern
4. factory_fn! - Com tipos explícitos
5. def_factory! - Mais simples
6. define_factory! - VERSÃO FINAL COMPLETA ✨

RECOMENDAÇÃO:
Use define_factory! - ela cria:
- Função factory simples: create_specialty(&ctx)
- Builder pattern: CreateSpecialtyBuilder::new().name(...).create(&ctx)
- Build sem DB: builder.build()
- Helper function: create_specialty_builder()

É a mais completa e flexível!
*/
