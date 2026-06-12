use actix_web::web;

use crate::handlers::{
    // base
    index,
    raw_index,

    //about,
    toggle_language,
    toggle_language_index,
    toggle_language_two,
    toggle_language_three,

    // admin
    // errors
    internal_server_error,
    not_found,

    // login
    login_form_input,
    login_handler,

    // person
    person_by_id,
    person_by_name,

    // role
    role_by_id,

    // capability
    capability_search,

    // organization
    organization_by_id,
    create_organization_form,
    create_organization_post,
    edit_organization_form,
    edit_organization_post,
    retire_organization_form,
    retire_organization_post,

    // org_tier
    org_tier_by_id,
    create_org_tier_form,
    create_org_tier_post,
    edit_org_tier_form,
    edit_org_tier_post,
    retire_org_tier_form,
    retire_org_tier_post,

    // org chart builder
    org_chart_builder,
    org_tier_node_partial,
    org_tier_panel_partial,

    // team
    team_by_id,
    
    // publication
    publication_by_id,

    // work
    work_by_id,

    // task
    task_by_id,

};

pub fn configure_services(config: &mut web::ServiceConfig) {
    config.service(index);
    config.service(raw_index);

    // login
    config.service(login_handler);
    config.service(login_form_input);

    // person
    config.service(person_by_id);
    config.service(person_by_name);

    // role
    config.service(role_by_id);

    // capability
    config.service(capability_search);

    // organization
    // "new" must be registered before the {organization_id} catch-all
    config.service(create_organization_form);
    config.service(create_organization_post);
    config.service(edit_organization_form);
    config.service(edit_organization_post);
    config.service(retire_organization_form);
    config.service(retire_organization_post);
    config.service(organization_by_id);

    // org_tier
    // "new" must be registered before the {org_tier_id} catch-all
    config.service(create_org_tier_form);
    config.service(create_org_tier_post);
    config.service(edit_org_tier_form);
    config.service(edit_org_tier_post);
    config.service(retire_org_tier_form);
    config.service(retire_org_tier_post);
    config.service(org_tier_by_id);

    // org chart builder
    config.service(org_chart_builder);
    config.service(org_tier_node_partial);
    config.service(org_tier_panel_partial);

    // team
    config.service(team_by_id);

    // publication
    config.service(publication_by_id);

    // work
    config.service(work_by_id);

    // task
    config.service(task_by_id);
    
    //config.service(about);
    config.service(toggle_language);
    config.service(toggle_language_index);
    config.service(toggle_language_two);
    config.service(toggle_language_three);

    // errors
    config.service(internal_server_error);
    config.service(not_found);

}
