use std::collections::HashMap;

use quote::quote;
use syn::Attribute;
use syn::{parse_macro_input, DeriveInput, AttrStyle::Inner};

fn extract_diesel_table_name(attr: &Attribute) -> Option<proc_macro2::TokenStream> {
    if let Inner(_bang) = attr.style { return None; }
    if attr.path.segments.is_empty() { return None; }
    if attr.path.segments[0].ident.to_string() != "diesel".to_string() { return None; }
    let mut token_iterator = attr.tokens.clone().into_iter();
    match token_iterator.next() {
        Some(token) => match token {
            proc_macro2::TokenTree::Group(group) => {
                if let proc_macro2::Delimiter::Parenthesis = group.delimiter() {} else { return None; }
                let mut stream_iterator = group.stream().into_iter();
                match stream_iterator.next() {
                    None => return None,
                    Some(token) => match token {
                        proc_macro2::TokenTree::Ident(ident) => {
                            if ident.to_string() != "table_name".to_string() { return None; }
                        },
                        _ => return None
                    }
                };
                match stream_iterator.next() {
                    None => return None,
                    Some(token) => match token {
                        proc_macro2::TokenTree::Punct(punct) => {
                            if punct.as_char() != '=' { return None; }
                        }
                        _ => return None
                    }
                };
                let mut result = proc_macro2::TokenStream::new();
                result.extend(stream_iterator);
                Some(result)
            },
            _ => return None
        }
        None => return None
    }
}

fn derive_rest_new(input: proc_macro::TokenStream, _pre: bool, _post: bool, _app_data: &proc_macro2::TokenStream, _connection: &proc_macro2::TokenStream) -> proc_macro::TokenStream {
    let parsed: DeriveInput = parse_macro_input!(input);
    let mut new_struct = parsed.clone();
    let diesel_table_name = match new_struct
        .attrs
        .iter()
        .flat_map(|attr| {
            extract_diesel_table_name(attr)
        })
        .next() {
            None => panic!("NewRest macro requires an #[diesel(table_name=<identifier>)] attribute"),
            Some(name) => name
        };
    new_struct.attrs = Vec::new();
    new_struct.ident = syn::Ident::new( &format!("New{}", new_struct.ident.to_string()), new_struct.ident.span());

    match &mut new_struct.data {
        syn::Data::Struct(data_struct) => {
            match &mut data_struct.fields {
                syn::Fields::Named(data_struct_fields) => {
                    data_struct_fields.named = 
                        data_struct_fields
                            .named
                            .pairs()
                            .filter(|p| {
                                p.value().ident.clone().unwrap_or_else(|| syn::Ident::new("", proc_macro2::Span::call_site())).to_string() != "id".to_string()
                            })
                            .fold(syn::punctuated::Punctuated::<syn::Field, syn::token::Comma>::new(), |mut punctuated, pair| {
                                let field = pair.value().clone().clone();
                                punctuated.push(field);
                                punctuated
                            });
                },
                _ => panic!("Macro can only be used in named fields struct types")
            }
        }
        _ => panic!("Macro can only be used in struct types")
    }
    let output = quote!{
        #[derive(Debug, Clone, serde::Deserialize, Insertable)]
        #[diesel(table_name = #diesel_table_name )]
        #new_struct
    };
    output.into()
}

fn derive_rest_collection(input: proc_macro::TokenStream, _pre: bool, _post: bool, app_data: &proc_macro2::TokenStream, connection: &proc_macro2::TokenStream) -> proc_macro::TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let output = quote! {
        #[async_trait]
        impl RestCollection<rest::RestCollectionGetParameters, #app_data, #connection> for #ident {
            async fn get(app_data: actix_web::web::Data<#app_data>, actix_web::web::Query(query_parameters): actix_web::web::Query<rest::RestCollectionGetParameters>) -> negotiated::Responder<#app_data> {
                actix_web::web::block(move || -> negotiated::Responder<#app_data> {
                    let mut db = <#app_data as rest::DbFactory<#connection>>::db(&app_data);
                    #ident::db_fetch_all(&mut db, query_parameters.q.unwrap_or_default(), None).into()
                }).await.into()
            }
        }
    };
    output.into()
}
fn derive_rest(input: proc_macro::TokenStream, pre: bool, post: bool, app_data: &proc_macro2::TokenStream, connection: &proc_macro2::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);
    let DeriveInput { ident, .. } = input;
    let new_ident = syn::Ident::new( &format!("New{}", ident.to_string()), ident.span() );
    let output = match (pre, post) {
        (true, true) => quote! {
            #[async_trait]
            impl Rest<#ident, #new_ident, #app_data, #connection> for #ident
            where #ident: rest::RestPre<#ident, #new_ident, #app_data> + rest::RestPost<#ident, #new_ident>
            {
                async fn post(app_data: actix_web::web::Data<#app_data>, actix_web::web::Json(host_group): actix_web::web::Json<#new_ident>) -> negotiated::Responder<#app_data> {
                    actix_web::web::block(move || -> negotiated::Responder<#app_data> {
                        let mut db = <#app_data as rest::DbFactory<#connection>>::db(&app_data);
                        <#ident as rest::RestPost<#ident, #new_ident>>::post_post(
                            &host_group,
                            <#ident as rest::Crud<#ident, #new_ident, #connection>>::db_insert(
                                &mut db,
                                &match <#ident as rest::RestPre<#ident, #new_ident, #app_data>>::pre_post(&app_data, &host_group) {
                                    Ok(host_group) => host_group,
                                    Err(err) => return err.into()
                                }
                            ).into()
                        ).into()
                    }).await.into()
                }
                async fn get(app_data: actix_web::web::Data<#app_data>, id: actix_web::web::Path<i32>) -> negotiated::Responder<#app_data> {
                    actix_web::web::block(move || -> negotiated::Responder<#app_data> {
                        let mut db = <#app_data as rest::DbFactory<#connection>>::db(&app_data);
                        <#ident as rest::RestPost<#ident, #new_ident>>::post_get(
                            id.clone(),
                            <#ident as rest::Crud<#ident, #new_ident, #connection>>::db_fetch(
                                &mut db,
                                match <#ident as rest::RestPre<#ident, #new_ident, #app_data>>::pre_get(&app_data, id.into_inner().try_into().unwrap()) {
                                    Ok(id) => id,
                                    Err(err) => return err.into()
                                }
                            ).into()
                        ).into()
                    }).await.into()
                }
                async fn put(app_data: actix_web::web::Data<#app_data>, id: actix_web::web::Path<i32>, to_update: actix_web::web::Json<#ident>) -> negotiated::Responder<#app_data> {
                    actix_web::web::block(move || -> negotiated::Responder<#app_data> {
                        let mut db = <#app_data as rest::DbFactory<#connection>>::db(&app_data);
                        let (filtered_id, filtered_to_update) = match <#ident as rest::RestPre<#ident, #new_ident, #app_data>>::pre_put(&app_data, id.clone(), &to_update) {
                            Ok(tuple) => tuple,
                            Err(err) => return err.into()
                        };
                        let mut merged_to_update = filtered_to_update.clone();
                        merged_to_update.id = id.clone();
                        <#ident as rest::RestPost<#ident, #new_ident>>::post_put(
                            id.clone(),
                            &to_update,
                            <#ident as rest::Crud<#ident, #new_ident, #connection>>::db_update(&mut db, &merged_to_update).into()
                        ).into()
                    }).await.into()
                }
                async fn delete(app_data: actix_web::web::Data<#app_data>, id: actix_web::web::Path<i32>) -> negotiated::Responder<#app_data> {
                    actix_web::web::block(move || -> negotiated::Responder<#app_data> {
                        let mut db = <#app_data as rest::DbFactory<#connection>>::db(&app_data);
                        <#ident as rest::RestPost<#ident, #new_ident>>::post_delete(
                            id.clone(),
                            <#ident as rest::Crud<#ident, #new_ident, #connection>>::db_delete(
                                &mut db,
                                match <#ident as rest::RestPre<#ident, #new_ident, #app_data>>::pre_delete(&app_data, id.into_inner().try_into().unwrap()) {
                                    Ok(id) => id,
                                    Err(err) => return err.into()
                                }
                            ).into()
                        ).into()
                    }).await.into()
                }
            }
        },
        (true, false) => quote! {
            #[async_trait]
            impl Rest<#ident, #new_ident, #app_data, #connection> for #ident
            where #ident: rest::RestPre<#ident, #new_ident, #app_data>
            {
                async fn post(app_data: actix_web::web::Data<#app_data>, actix_web::web::Json(host_group): actix_web::web::Json<#new_ident>) -> negotiated::Responder<#app_data> {
                    actix_web::web::block(move || -> negotiated::Responder<#app_data> {
                        let mut db = <#app_data as rest::DbFactory<#connection>>::db(&app_data);
                        <#ident as rest::Crud<#ident, #new_ident, #connection>>::db_insert(
                            &mut db,
                            &match <#ident as rest::RestPre<#ident, #new_ident, #app_data>>::pre_post(&app_data, &host_group) {
                                Ok(host_group) => host_group,
                                Err(err) => return err.into()
                            }
                        ).into()
                    }).await.into()
                }
                async fn get(app_data: actix_web::web::Data<#app_data>, id: actix_web::web::Path<i32>) -> negotiated::Responder<#app_data> {
                    actix_web::web::block(move || -> negotiated::Responder<#app_data> {
                        let mut db = <#app_data as rest::DbFactory<#connection>>::db(&app_data);
                        <#ident as rest::Crud<#ident, #new_ident, #connection>>::db_fetch(
                            &mut db,
                            match <#ident as rest::RestPre<#ident, #new_ident, #app_data>>::pre_get(&app_data, id.into_inner().try_into().unwrap()) {
                                Ok(id) => id,
                                Err(err) => return err.into()
                            }
                        ).into()
                    }).await.into()
                }
                async fn put(app_data: actix_web::web::Data<#app_data>, id: actix_web::web::Path<i32>, to_update: actix_web::web::Json<#ident>) -> negotiated::Responder<#app_data> {
                    actix_web::web::block(move || -> negotiated::Responder<#app_data> {
                        let mut db = <#app_data as rest::DbFactory<#connection>>::db(&app_data);
                        let (filtered_id, filtered_to_update) = match <#ident as rest::RestPre<#ident, #new_ident, #app_data>>::pre_put(&app_data, id.clone(), &to_update) {
                            Ok(tuple) => tuple,
                            Err(err) => return err.into()
                        };
                        let mut merged_to_update = filtered_to_update.clone();
                        merged_to_update.id = id.clone();
                        <#ident as rest::Crud<#ident, #new_ident, #connection>>::db_update(&mut db, &merged_to_update).into()
                    }).await.into()
                }
                async fn delete(app_data: actix_web::web::Data<#app_data>, id: actix_web::web::Path<i32>) -> negotiated::Responder<#app_data> {
                    actix_web::web::block(move || -> negotiated::Responder<#app_data> {
                        let mut db = <#app_data as rest::DbFactory<#connection>>::db(&app_data);
                        <#ident as rest::Crud<#ident, #new_ident, #connection>>::db_delete(
                            &mut db,
                            match <#ident as rest::RestPre<#ident, #new_ident, #app_data>>::pre_delete(&app_data, id.into_inner().try_into().unwrap()) {
                                Ok(id) => id,
                                Err(err) => return err.into()
                            }
                        ).into()
                    }).await.into()
                }
            }
        },
        (false, true) => quote! {
            #[async_trait]
            impl Rest<#ident, #new_ident, #app_data, #connection> for #ident
            where #ident: rest::RestPost<#ident, #new_ident>
            {
                async fn post(app_data: actix_web::web::Data<#app_data>, actix_web::web::Json(host_group): actix_web::web::Json<#new_ident>) -> negotiated::Responder<#app_data> {
                    actix_web::web::block(move || -> negotiated::Responder<#app_data> {
                        let mut db = <#app_data as rest::DbFactory<#connection>>::db(&app_data);
                        <#ident as rest::RestPost<#ident, #new_ident>>::post_post(
                            &host_group,
                            <#ident as rest::Crud<#ident, #new_ident, #connection>>::db_insert(&mut db, &host_group).into()
                        ).into()
                    }).await.into()
                }
                async fn get(app_data: actix_web::web::Data<#app_data>, id: actix_web::web::Path<i32>) -> negotiated::Responder<#app_data> {
                    actix_web::web::block(move || -> negotiated::Responder<#app_data> {
                        let mut db = <#app_data as rest::DbFactory<#connection>>::db(&app_data);
                        <#ident as rest::RestPost<#ident, #new_ident>>::post_get(
                            id.clone(),
                            <#ident as rest::Crud<#ident, #new_ident, #connection>>::db_fetch(&mut db, id.into_inner().try_into().unwrap()).into()
                        ).into()
                    }).await.into()
                }
                async fn put(app_data: actix_web::web::Data<#app_data>, id: actix_web::web::Path<i32>, to_update: actix_web::web::Json<#ident>) -> negotiated::Responder<#app_data> {
                    actix_web::web::block(move || -> negotiated::Responder<#app_data> {
                        let mut db = <#app_data as rest::DbFactory<#connection>>::db(&app_data);
                        let mut merged_to_update = to_update.clone();
                        merged_to_update.id = id.clone();
                        <#ident as rest::RestPost<#ident, #new_ident>>::post_put(
                            id.clone(),
                            &to_update,
                            <#ident as rest::Crud<#ident, #new_ident, #connection>>::db_update(&mut db, &merged_to_update).into()
                        ).into()
                    }).await.into()
                }
                async fn delete(app_data: actix_web::web::Data<#app_data>, id: actix_web::web::Path<i32>) -> negotiated::Responder<#app_data> {
                    actix_web::web::block(move || -> negotiated::Responder<#app_data> {
                        let mut db = <#app_data as rest::DbFactory<#connection>>::db(&app_data);
                        <#ident as rest::RestPost<#ident, #new_ident>>::post_delete(
                            id.clone(),
                            <#ident as rest::Crud<#ident, #new_ident, #connection>>::db_delete(&mut db, id.into_inner().try_into().unwrap()).into()
                        ).into()
                    }).await.into()
                }
            }
        },
        (false, false) => quote! {
            #[async_trait]
            impl Rest<#ident, #new_ident, #app_data, #connection> for #ident {
                async fn post(app_data: actix_web::web::Data<#app_data>, actix_web::web::Json(host_group): actix_web::web::Json<#new_ident>) -> negotiated::Responder<#app_data> {
                    actix_web::web::block(move || -> negotiated::Responder<#app_data> {
                        let mut db = <#app_data as rest::DbFactory<#connection>>::db(&app_data);
                        <#ident as rest::Crud<#ident, #new_ident, #connection>>::db_insert(&mut db, &host_group).into()
                    }).await.into()
                }
                async fn get(app_data: actix_web::web::Data<#app_data>, id: actix_web::web::Path<i32>) -> negotiated::Responder<#app_data> {
                    actix_web::web::block(move || -> negotiated::Responder<#app_data> {
                        let mut db = <#app_data as rest::DbFactory<#connection>>::db(&app_data);
                        <#ident as rest::Crud<#ident, #new_ident, #connection>>::db_fetch(&mut db, id.into_inner().try_into().unwrap()).into()
                    }).await.into()
                }
                async fn put(app_data: actix_web::web::Data<#app_data>, id: actix_web::web::Path<i32>, to_update: actix_web::web::Json<#ident>) -> negotiated::Responder<#app_data> {
                    actix_web::web::block(move || -> negotiated::Responder<#app_data> {
                        let mut db = <#app_data as rest::DbFactory<#connection>>::db(&app_data);
                        let mut merged_to_update = to_update.clone();
                        merged_to_update.id = id.clone();
                        <#ident as rest::Crud<#ident, #new_ident, #connection>>::db_update(&mut db, &merged_to_update).into()
                    }).await.into()
                }
                async fn delete(app_data: actix_web::web::Data<#app_data>, id: actix_web::web::Path<i32>) -> negotiated::Responder<#app_data> {
                    actix_web::web::block(move || -> negotiated::Responder<#app_data> {
                        let mut db = <#app_data as rest::DbFactory<#connection>>::db(&app_data);
                        <#ident as rest::Crud<#ident, #new_ident, #connection>>::db_delete(&mut db, id.into_inner().try_into().unwrap()).into()
                    }).await.into()
                }
            }
        },
    };
    output.into()
}

fn hash_attributes(attrs: &Vec<Attribute>, ident: &str) -> HashMap<String, proc_macro2::TokenStream> {
    attrs
        .iter()
        .filter(|attr| attr.path.is_ident(ident) )
        .map(|attr| &attr.tokens )
        .fold(HashMap::<String, Vec::<proc_macro2::TokenTree>>::new(), |result, attr_tokens| {
            match attr_tokens.clone().into_iter().next() {
                Some(proc_macro2::TokenTree::Group(group)) => group
                        .stream()
                        .into_iter()
                        .fold((result, "".to_string()), |(mut result, current_ident): (HashMap<String, Vec::<proc_macro2::TokenTree>>, String), attr_token| {
                    match &attr_token {
                        proc_macro2::TokenTree::Ident(ident) => {
                            if current_ident.len() == 0 {
                                result.insert(ident.to_string(), Vec::new());
                                return (result, ident.to_string());
                            } else {
                                result.get_mut(&current_ident).unwrap().push(proc_macro2::TokenTree::Ident(ident.clone()));
                            }
                        }
                        proc_macro2::TokenTree::Punct( punct ) => {
                            if punct.as_char() == ',' { return (result, "".to_string()); }
                            if punct.as_char() != '=' || result.get(&current_ident).unwrap().len() > 0 { result.get_mut(&current_ident).unwrap().push(attr_token.clone()); }
                        },
                        other => {
                            result.get_mut(&current_ident).unwrap().push(other.clone());
                        }
                    };
                    (result, current_ident)
                }).0,
                _ => panic!("Unexpected token in rest attributes")
            }
        })
        .into_iter()
        .fold(HashMap::<String, proc_macro2::TokenStream>::new(), |mut result, (k, token_tree_vec)| {
            result.insert(k.clone(), proc_macro2::TokenStream::from_iter(token_tree_vec));
            result
        })
}

#[proc_macro_derive(Rest, attributes(rest))]
pub fn derive_rest_traits(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input_to_derive = input.clone();
    let derive_input = parse_macro_input!(input_to_derive);
    let DeriveInput {
        attrs, ..
    } = derive_input;
    let attribute_hash = hash_attributes(&attrs, "rest");
    let attribute_pre = match attribute_hash.get("pre").expect("pre attribute is mandatory in Rest derive").clone().into_iter().next().expect("pre attribute has no value in Rest derive") {
        proc_macro2::TokenTree::Ident(ident) => { ident.to_string() == "true".to_string() },
        _ => panic!("Unexpected value for pre attribute in Rest derive"),
    };
    let attribute_post = match attribute_hash.get("post").expect("post attribute is mandatory in Rest derive").clone().into_iter().next().expect("post attribute has no value in Rest derive") {
        proc_macro2::TokenTree::Ident(ident) => { ident.to_string() == "true".to_string() },
        _ => panic!("Unexpected value for post attribute in Rest derive"),
    };
    let attribute_app_data = attribute_hash.get("app_data").expect("pre attribute is mandatory in Rest derive");
    let attribute_connection = attribute_hash.get("connection").expect("connection attribute is mandatory in Rest derive");
    let mut result = proc_macro::TokenStream::new();
    result.extend(derive_rest_new(input.clone(), attribute_pre, attribute_post, attribute_app_data, attribute_connection));
    result.extend(derive_rest_collection(input.clone(), attribute_pre, attribute_post, attribute_app_data, attribute_connection));
    result.extend(derive_rest(input.clone(), attribute_pre, attribute_post, attribute_app_data, attribute_connection));
    result
}

#[proc_macro_derive(Crud, attributes(crud))]
pub fn derive_crud(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input_to_derive = input.clone();
    let derive_input = parse_macro_input!(input_to_derive);
    let DeriveInput {
        ident,
        attrs, ..
    } = derive_input;
    let attribute_hash = hash_attributes(&attrs, "crud");
    let table_name = attribute_hash.get("table_name").expect("table_name attribute is mandatory in Crud derive");
    let connection = attribute_hash.get("connection").expect("connection attribute is mandatory in Crud derive");
    let new_ident = proc_macro2::Ident::new( &format!("New{}", ident.to_string()), ident.span().clone() );
    let default_search_field = proc_macro2::TokenStream::from_iter(vec![proc_macro2::TokenTree::Ident(proc_macro2::Ident::new("name", proc_macro2::Span::call_site()))].into_iter());
    let search_field = if attribute_hash.contains_key("search_field") { 
        attribute_hash.get("search_field").unwrap()
    } else {
        &default_search_field
    };
    println!("{:?}", search_field);
    let output = quote!{
        impl rest::Crud<#ident, #new_ident, #connection> for #ident {
            fn db_insert(db: &mut #connection, to_insert: &#new_ident) -> anyhow::Result<#ident> {
                Ok(diesel::insert_into(#table_name::table)
                    .values(to_insert)
                    .get_result::<#ident>(db)?)
            }
            fn db_update(db: &mut #connection, to_update: &#ident) -> anyhow::Result<#ident> {
                Ok(diesel::update(#table_name::table)
                    .filter(crate::#table_name::dsl::id.eq(to_update.id))
                    .set(to_update)
                    .get_result::<#ident>(db)?)
            }
            fn db_fetch_all(db: &mut #connection, text_filter: String, limit: Option<(i64, i64)>) -> anyhow::Result<Vec<#ident>> {
                let mut query = #table_name::table.into_boxed();
                if !text_filter.is_empty() { query = query.filter(#table_name::#search_field.like(format!("%{}%", text_filter).to_string())); }
                if let Some(limit_value) = limit { query = query.limit(limit_value.0).offset(limit_value.1); }
                Ok(query.load::<#ident>(db)?)
            }
            fn db_fetch(db: &mut #connection, id: i32) -> anyhow::Result<#ident> {
                Ok(#table_name::table
                    .filter(crate::schema::host_group::dsl::id.eq(id))
                    .first(db)?)
            }
            fn db_delete(db: &mut #connection, id: i32) -> anyhow::Result<#ident> {
                let result = Self::db_fetch(db, id);
                diesel::delete(#table_name::table)
                    .filter(crate::#table_name::dsl::id.eq(id))
                    .execute(db)?;
                result
            }
        }
    };
    output.into()
}