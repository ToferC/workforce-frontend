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
    person_index,
    create_person_form,
    create_person_post,
    edit_person_form,
    edit_person_post,
    retire_person_form,
    retire_person_post,
    create_affiliation_form,
    create_affiliation_post,
    end_affiliation_post,
    create_language_form,
    create_language_post,

    // role
    role_by_id,
    role_index,
    create_role_form,
    create_role_post,
    edit_role_form,
    edit_role_post,
    end_role_form,
    end_role_post,
    create_requirement_form,
    create_requirement_post,
    retire_requirement_post,

    // capability
    capability_search,
    create_capability_form,
    create_capability_post,
    retire_capability_post,
    validate_capability_form,
    validate_capability_post,

    // skill
    skill_index,
    skill_by_id,
    create_skill_form,
    create_skill_post,
    edit_skill_form,
    edit_skill_post,

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
    assign_org_owner_form,
    assign_org_owner_post,

    // org chart builder
    org_chart_builder,
    org_tier_node_partial,
    org_tier_panel_partial,

    // team
    team_by_id,
    team_index,
    create_team_form,
    create_team_post,
    edit_team_form,
    edit_team_post,
    retire_team_form,
    retire_team_post,
    assign_team_owner_form,
    assign_team_owner_post,
    
    // publication
    publication_by_id,
    publication_index,
    create_publication_form,
    create_publication_post,
    edit_publication_form,
    edit_publication_post,

    // work
    work_by_id,
    create_work_form,
    create_work_post,
    edit_work_form,
    edit_work_post,

    // task
    task_by_id,
    task_index,
    create_task_form,
    create_task_post,
    edit_task_form,
    edit_task_post,

};

pub fn configure_services(config: &mut web::ServiceConfig) {
    config.service(index);
    config.service(raw_index);

    // login
    config.service(login_handler);
    config.service(login_form_input);

    // person
    // "new" must be registered before the {person_id} catch-all
    config.service(create_person_form);
    config.service(create_person_post);
    config.service(edit_person_form);
    config.service(edit_person_post);
    config.service(retire_person_form);
    config.service(retire_person_post);
    config.service(create_affiliation_form);
    config.service(create_affiliation_post);
    config.service(end_affiliation_post);
    config.service(create_language_form);
    config.service(create_language_post);
    config.service(person_index);
    config.service(person_by_id);
    config.service(person_by_name);

    // role
    // "new" must be registered before the {role_id} catch-all
    config.service(create_role_form);
    config.service(create_role_post);
    config.service(edit_role_form);
    config.service(edit_role_post);
    config.service(end_role_form);
    config.service(end_role_post);
    config.service(role_index);
    config.service(create_requirement_form);
    config.service(create_requirement_post);
    config.service(retire_requirement_post);
    config.service(role_by_id);

    // capability
    config.service(capability_search);
    config.service(create_capability_form);
    config.service(create_capability_post);
    config.service(retire_capability_post);
    config.service(validate_capability_form);
    config.service(validate_capability_post);

    // skill
    // "new" must be registered before the {skill_id} catch-all
    config.service(skill_index);
    config.service(create_skill_form);
    config.service(create_skill_post);
    config.service(edit_skill_form);
    config.service(edit_skill_post);
    config.service(skill_by_id);

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
    config.service(assign_org_owner_form);
    config.service(assign_org_owner_post);
    config.service(org_tier_by_id);

    // org chart builder
    config.service(org_chart_builder);
    config.service(org_tier_node_partial);
    config.service(org_tier_panel_partial);

    // team
    // "new" must be registered before the {team_id} catch-all
    config.service(create_team_form);
    config.service(create_team_post);
    config.service(edit_team_form);
    config.service(edit_team_post);
    config.service(retire_team_form);
    config.service(retire_team_post);
    config.service(assign_team_owner_form);
    config.service(assign_team_owner_post);
    config.service(team_index);
    config.service(team_by_id);

    // publication
    // "new" before {publication_id} catch-all
    config.service(publication_index);
    config.service(create_publication_form);
    config.service(create_publication_post);
    config.service(edit_publication_form);
    config.service(edit_publication_post);
    config.service(publication_by_id);

    // work
    config.service(create_work_form);
    config.service(create_work_post);
    config.service(edit_work_form);
    config.service(edit_work_post);
    config.service(work_by_id);

    // task
    config.service(task_index);
    config.service(create_task_form);
    config.service(create_task_post);
    config.service(edit_task_form);
    config.service(edit_task_post);
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
