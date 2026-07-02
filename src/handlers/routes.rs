use actix_web::web;

use crate::handlers::{
    // base
    index,
    raw_index,

    about,
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

    // account lifecycle / self-service
    activate_form,
    activate_post,
    my_profile,
    my_profile_post,
    flag_issue_post,
    grant_access_post,

    // admin portal
    admin_users,
    admin_user_new_form,
    admin_user_create,
    admin_user_edit_form,
    admin_user_update,
    admin_user_invite,
    admin_user_disable,
    admin_user_enable,
    admin_flags,
    admin_flag_resolve,

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
    restore_person_post,
    create_affiliation_form,
    create_affiliation_post,
    end_affiliation_post,
    create_language_form,
    create_language_post,

    // role
    role_by_id,
    role_matches,
    role_index,
    create_role_form,
    create_role_post,
    edit_role_form,
    edit_role_post,
    end_role_form,
    end_role_post,
    assign_role_post,
    transfer_preview,
    vacate_role_post,
    offer_role_post,
    create_requirement_form,
    create_requirement_post,
    retire_requirement_post,
    edit_requirement_form,
    edit_requirement_post,

    // capability
    capability_search,
    create_capability_form,
    create_capability_post,
    retire_capability_post,
    validate_capability_form,
    validate_capability_post,

    // skill
    skill_index,
    skill_options,
    skill_by_id,
    create_skill_form,
    create_skill_post,
    edit_skill_form,
    edit_skill_post,

    // organization
    organization_index,
    organization_by_id,
    create_organization_form,
    create_organization_post,
    edit_organization_form,
    edit_organization_post,
    retire_organization_form,
    retire_organization_post,
    restore_organization_post,

    // org_tier
    org_tier_index,
    org_tier_by_id,
    create_org_tier_form,
    create_org_tier_post,
    edit_org_tier_form,
    edit_org_tier_post,
    retire_org_tier_form,
    retire_org_tier_post,
    restore_org_tier_post,
    assign_org_owner_form,
    assign_org_owner_post,

    // org chart builder
    org_chart_builder,
    org_chart_explore,
    team_members_json,
    org_tier_node_partial,
    org_tier_panel_partial,

    // manager panel
    manage_panel,
    accept_offer_post,
    decline_offer_post,
    withdraw_offer_post,
    activity_view,

    // team
    team_by_id,
    team_index,
    create_team_form,
    create_team_post,
    edit_team_form,
    edit_team_post,
    retire_team_form,
    retire_team_post,
    restore_team_post,
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
    work_index,
    work_skill_options,
    vacancies,
    create_work_form,
    create_work_post,
    create_vacant_work_form,
    create_vacant_work_post,
    edit_work_form,
    edit_work_post,
    assign_work_form,
    assign_work_post,

    // task
    task_by_id,
    task_index,
    create_task_form,
    create_task_post,
    create_product_task_form,
    create_team_task_form,
    create_team_task_post,
    edit_task_form,
    edit_task_post,

    // product
    product_by_id,
    product_index,
    create_product_form,
    create_product_post,
    edit_product_form,
    edit_product_post,

    // analytics
    analytics_dashboard,
    analytics_section_work,
    analytics_section_capacity,
    analytics_section_vacancies,
    analytics_section_gaps,
    analytics_coverage,
    analytics_delivery,
    analytics_mobility_view,
    analytics_growth,
    analytics_supply_demand,

};

pub fn configure_services(config: &mut web::ServiceConfig) {
    config.service(index);
    config.service(raw_index);

    // login
    config.service(login_handler);
    config.service(login_form_input);

    // account lifecycle / self-service
    config.service(activate_form);
    config.service(activate_post);
    config.service(my_profile);
    config.service(my_profile_post);
    config.service(flag_issue_post);
    config.service(grant_access_post);

    // admin portal — "new" must be registered before the {user_id} catch-alls
    config.service(admin_users);
    config.service(admin_user_new_form);
    config.service(admin_user_create);
    config.service(admin_user_edit_form);
    config.service(admin_user_update);
    config.service(admin_user_invite);
    config.service(admin_user_disable);
    config.service(admin_user_enable);
    config.service(admin_flags);
    config.service(admin_flag_resolve);

    // person
    // "new" must be registered before the {person_id} catch-all
    config.service(create_person_form);
    config.service(create_person_post);
    config.service(edit_person_form);
    config.service(edit_person_post);
    config.service(retire_person_form);
    config.service(retire_person_post);
    config.service(restore_person_post);
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
    config.service(assign_role_post);
    config.service(transfer_preview);
    config.service(vacate_role_post);
    config.service(offer_role_post);
    config.service(role_index);
    config.service(create_requirement_form);
    config.service(create_requirement_post);
    config.service(edit_requirement_form);
    config.service(edit_requirement_post);
    config.service(retire_requirement_post);
    config.service(role_matches);
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
    config.service(skill_options);
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
    config.service(restore_organization_post);
    config.service(organization_index);
    config.service(organization_by_id);

    // org_tier
    // "new" and "index" must be registered before the {org_tier_id} catch-all
    config.service(org_tier_index);
    config.service(create_org_tier_form);
    config.service(create_org_tier_post);
    config.service(edit_org_tier_form);
    config.service(edit_org_tier_post);
    config.service(retire_org_tier_form);
    config.service(retire_org_tier_post);
    config.service(restore_org_tier_post);
    config.service(assign_org_owner_form);
    config.service(assign_org_owner_post);
    config.service(org_tier_by_id);

    // org chart builder
    config.service(org_chart_explore);
    config.service(team_members_json);
    config.service(org_chart_builder);
    config.service(org_tier_node_partial);
    config.service(org_tier_panel_partial);

    // manager panel
    config.service(manage_panel);
    config.service(accept_offer_post);
    config.service(decline_offer_post);
    config.service(withdraw_offer_post);
    config.service(activity_view);

    // team
    // "new" must be registered before the {team_id} catch-all
    config.service(create_team_form);
    config.service(create_team_post);
    config.service(edit_team_form);
    config.service(edit_team_post);
    config.service(retire_team_form);
    config.service(retire_team_post);
    config.service(restore_team_post);
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

    // work — specific sub-paths before {work_id} catch-all
    config.service(work_index);
    config.service(work_skill_options);
    config.service(vacancies);
    config.service(create_work_form);
    config.service(create_work_post);
    config.service(create_vacant_work_form);
    config.service(create_vacant_work_post);
    config.service(assign_work_form);
    config.service(assign_work_post);
    config.service(edit_work_form);
    config.service(edit_work_post);
    config.service(work_by_id);

    // task
    config.service(task_index);
    config.service(create_task_form);
    config.service(create_task_post);
    config.service(create_product_task_form);
    config.service(create_team_task_form);
    config.service(create_team_task_post);
    config.service(edit_task_form);
    config.service(edit_task_post);
    config.service(task_by_id);

    // product — "new" must be registered before the {product_id} catch-all
    config.service(product_index);
    config.service(create_product_form);
    config.service(create_product_post);
    config.service(edit_product_form);
    config.service(edit_product_post);
    config.service(product_by_id);

    // analytics — specific sub-paths before the dashboard catch-all
    config.service(analytics_coverage);
    config.service(analytics_delivery);
    config.service(analytics_mobility_view);
    config.service(analytics_growth);
    config.service(analytics_supply_demand);
    config.service(analytics_section_work);
    config.service(analytics_section_capacity);
    config.service(analytics_section_vacancies);
    config.service(analytics_section_gaps);
    config.service(analytics_dashboard);

    config.service(about);
    config.service(toggle_language);
    config.service(toggle_language_index);
    config.service(toggle_language_two);
    config.service(toggle_language_three);

    // errors
    config.service(internal_server_error);
    config.service(not_found);

}
