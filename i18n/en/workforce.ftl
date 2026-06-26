hello-world = Hello World
welcome-to = Welcome to the Armed Forces Workforce Analytics MVP

## General Section

epicenter = Workforce Analytics
-app-name = Workforce Analytics
app-description = { -app-name } is a learning project and experiment in workforce analytics.
app-mvp = Workforce Analytics MVP
-user-support-email = usersupport@intersectional-data.ca

## Organization
organization = Organization
organizations = Organizations
org_tier = Organization Tier
team = Team
work = Work
affiliation = Affiliation
person = Person
publication = Publication
role = Role
task = Task

## Base & Navbar
button-explore-demo-data = Explore demonstration data
about = About the Project
language-toggle = Français
main-navigation = Main navigation
skip-to-content = Skip to main content
home = Home
profile = Profile
logout = Logout
user-index = User Index
login = Log in
register = Register
source-code = Source code

## Footer
footer-1 = Please note that this work is a learning project and experiment in developing actionable workforce analytics.
available-licence = The { -app-name } project is available on GitHub under an MIT licence here:
developed-by = Developed by ToferC 2026

## About Page
about-lead = A learning project and experiment in actionable workforce analytics.
about-what-heading = What this is
about-what-body = { -app-name } is an experimental application for exploring how an organization can manage its people, roles, capabilities, teams and work in one place. It treats the workforce as connected data — capabilities, role requirements, effort and delivery — so capacity and skill gaps can be seen at a glance.
about-tech-heading = How it is built
about-tech-body = A Rust web application built with Actix-Web and the Tera templating engine, talking to a GraphQL analytics API. The bilingual interface is aligned with the Government of Canada Design System.
about-source-heading = Source code and licence
about-source-body = The project is open source and available on GitHub under an MIT licence. You are welcome to read the code, open issues and contribute.
about-contact-heading = Contact
about-contact-body = Questions or feedback? Reach the team at { -user-support-email }.

## Login Page
email-address = Email address
email-address-helper = Enter the email address you used to register.
email-placeholder = yourname@domain.com
password = Password
password-placeholder = Your password...
password-helper = Your password (at least 12 characters long).
forgot-password = I forgot my password

## Register Page
register-to-app = Register to { -app-name }
why-register = You need to register an account to create and manage documents.
register-email-helper = Enter the email address you want to use to log in.
user-name = User name
user-name-placeholder = Firstname Lastname
user-name-helper = The name you'd like to be called when using Data Docs.
email-coming-notice = After registering you will receive a code by email to verify your email address. Check your email and then enter the code on the next page.

## Registration Error
registration-error = Registration Error
problem-with-registration = There was a problem with your registration. You may have already registered with this email address or there may be a problem with the information you entered.
you-can-login = You can log in
here = here
or-try-register = or try to register again.

## Password Reset Sent
password-reset-sent = Password Reset Email Sent
password-reset-details = An email has been sent to you containing a link to reset your password. Clicking the link will bring to to a page where you can reset your password.

## Change Password
change-password-title = Change Password for { $useremail }
enter-new-password-below = Enter your new password below.
new-password = New Password
update-password = Update Password

## Email Verification
verify-email = Verify your email
enter-key = Please enter the code you received at { $useremail } to complete your registration.
code = Code
code-helper = Enter the five digit code you received at the email you used to register.
verify = Verify

## Request Password Reset
request-password-reset = Request password reset
reset-instructions = Enter the email address you used to register below. If you have an account, we will send you an email with a link to reset your password.
send-reset-email = Send Password Reset Email

## User

## User Page
user-profile-for = User profile for:
user-details = User details and account management.
account-options = Account Options
edit-as-admin = Edit User as Admin
change-username-email = Change username or email address
reset-password = Reset Password
your-managed-communities = Your managed communities
community = Community
community-colon = Community:
private = Private
open = Open
created = Created:
data-use = Data Use:
experiences = Experiences
avg-inclusivity = Avg. Inclusivity
details-button = Details
graph-button = Graph
edit-button = Edit
add-community-button = Add Community
must-verify = You need to verify your account before you can create communities.
send-email-verification = Send email verification

## Edit User
edit-user-title = Edit User { $user }
edit-user-explain = Update and edit your account details here.
email-change-notice = If you update your email address, you will receive a code by email to verify your new email address. Check your email and then enter the code on the next page.

## Admin Edit User
edit-user-admin-title = Edit User { $user } as admin
edit-user-admin-explain = Update and edit the user's account details here.
role-helper = Choose role and privileges for this user.
admin = Admin
validated = Validated
true = True
false = False
validated-helper = Choose to validate or invalidate the user.
update-button = Update

## Delete User
delete-user = Delete user
delete-user-explain = Continue here to delete your user profile.
delete-user-colon = Delete User:
delete-user-placeholder = User to delete...
delete-user-helper = Enter the user name of the user you would like to delete. This action is permanent.
delete-user-communities-explain = When this profile is deleted, all owned communities will also be deleted.
    Any experiences associated with the profile will become part of the global community, but will no longer be associated with the deleted communities or each other.
return-button = Return
communities = Communities

## User Index
user = User
active-users = Active users on { -app-name }
email = Email
link = Link

## Email Verification
email-registered-with = Your email address has been registered with
registered-in-error = If you think this is an error, please contact { -user-support-email }.
register-instructions = If this is you, and you would like to verify your account, please enter the code below on
verification-page = the verification page
your-code = Your code is:
time-limit = You have 60 minutes to enter this code to verify your account. If you need to, you can
request-another-code = request another code
thank-you = Thank you,

## Password reset request
password-reset-received = We received a request to reset your password on { -app-name }. If you think this is an error, please contact { -user-support-email }.
if-you-instructions = If this was you, You can reset your password through the following link.
from-login-screen = from the log in screen.

## Person
person-index = People
person-by-name = Person by Name

## Errors

## 404
page-not-found = Page Not Found: 404
you-requested = You requested:
does-not-exist = which doesn't exist in this application.
wrong-turn = Looks like you took a wrong turn.
go-home = Go back home?

## Internal Server Error
internal-server-error = Internal Server Error
having-problems = Looks like we're having a bit of a problem.

## Not Authorized
not-authorized = Not Authorized
not-authorized-explain = You're not authorized to complete this action.
go-back-or = You can go back or
return-main-page = return to the main page

## Record not found
record-not-found = Record not found
record-not-found-explain = The Record you are searching for is not available.
## Organization Forms
create-organization = Create Organization
edit-organization = Edit Organization
retire-organization = Retire Organization
retire-organization-confirm = Are you sure you want to retire this organization? It will be marked as retired and hidden from active listings, but its history is preserved.
new-organization-button = New Organization
name-english = Name (English)
name-french = Name (French)
acronym-english = Acronym (English)
acronym-french = Acronym (French)
organization-type = Organization Type
organization-type-helper = e.g. department, agency, partner
website-url = Website URL
save-button = Save
cancel-button = Cancel
confirm-retire-button = Yes, retire
retired = Retired

## Org Tier Forms
create-org-tier = Create Organization Tier
edit-org-tier = Edit Organization Tier
retire-org-tier = Retire Tier
retire-org-tier-confirm = Are you sure you want to retire this organization tier? It will be marked as retired but its history is preserved. Child tiers and teams are not changed.
tier-level = Tier Level
tier-level-helper = 1 is the top of the organization; higher numbers are deeper in the hierarchy.
primary-domain = Primary Domain
parent-tier = Parent Tier
none-top-tier = None (top-level tier)
owner = Owner
child-tiers = Child tiers
teams = Teams

## Org Chart Builder
org-chart = Org Chart
org-chart-explore = Org Chart Explorer
org-chart-explore-help = Click a tier to expand or collapse it. Drag to pan and scroll to zoom. Teams appear under their tier.
how-to-explore = How to explore
visual-view = Visual view
list-view = List view
cap-legend = Team capacity (active effort)
cap-empty = Empty / no load
cap-light = Light
cap-moderate = Moderate
cap-heavy = Heavy
org-chart-help = Click a tier's info button to see its details here. Expand tiers on the right to explore child tiers, teams, roles, and the people in them.
back-to-organization = Back to organization
tier-info = Tier info
tier-count = Tiers
no-org-tiers = This organization has no tiers yet.
add-top-tier = Add top-level tier
add-child-tier = Add child tier
empty-tier = No child tiers or teams under this tier.
loading = Loading...
occupied = occupied
vacant = Vacant
filled = Filled
in-role = In role
available = Available
all-organizations = All organizations
all-statuses = All
no-roles = No roles in this team.

## Team Forms
create-team = Create Team
edit-team = Edit Team
retire-team = Retire Team
retire-team-confirm = Are you sure you want to retire this team? It will be marked as retired but its history and roles are preserved.
description-english = Description (English)
description-french = Description (French)
org-tier = Organization Tier
keep-current-domain = Keep current domain
add-team = Add team

## Person Forms
create-person = Create Person
edit-person = Edit Person
retire-person = Retire Person
retire-person-confirm = Are you sure you want to retire this person? Their record will be marked as retired but their history is preserved.
new-person-button = New Person
user-account-email = User account email
user-account-email-helper = The email of the registered user account this person record will be linked to.
given-name = Given name
family-name = Family name
work-email = Work email
phone = Phone
work-address = Work address
city = City
province = Province
postal-code = Postal code
country = Country
peoplesoft-id = PeopleSoft ID
orcid-id = ORCID iD
personnel-type = Personnel type
occupational-group = Occupational group
occupational-level = Occupational level
active-effort = Active effort

## Role Forms
create-role = Create Role
add-role = Add role
edit-role-status = Edit Role Status
end-role = Retire Position
end-role-confirm = Retire this position? It becomes inactive with an end date of today and is removed from active and vacancy lists. Do this only when the organization no longer needs the position — to record that the current person is leaving, use Vacate instead, which keeps the position. Assignment history is preserved either way.
confirm-end-role-button = Yes, retire position
role-ended = Retired
role-active = Role is active
role-edit-limited = Only the active flag and dates can be changed on an existing role. To change the title, rank, or person, end this role and create a new one — that preserves the history.
title-english = Title (English)
title-french = Title (French)
rank = Rank
military-occupation = Military Occupation
military-classification = Military classification (leave blank for civilian roles)
civilian-classification = Civilian classification (leave blank for military roles)
none-option = — None —
effort = Effort
effort-helper = Expected workload for this role; full time is around 10.
start-date = Start date
end-date = End date
assign-person = Assign person (optional)
assign-person-helper = Full given and family name of an existing person. Leave blank to create a vacant role.

## Link Entities (ownership, affiliation)
assign-owner = Assign owner
owner-name = Owning role
owner-name-helper = The manager position that owns this. Authority follows the position, not the person who currently holds it.
add-affiliation = Add affiliation
end-affiliation = End
affiliated-organization = Affiliated organization
affiliation-role = Affiliation role
affiliation-role-helper = e.g. Secondment, Liaison, Advisor.
none-label = None

## Skills & Capabilities
skills = Skills
skill = Skill
description = Description
create-skill = Create Skill
edit-skill = Edit Skill
new-skill-button = New Skill
people-with-skill = People with this skill
add-capability = Add capability
self-identified-level = Self-identified level
retire-button = Retire

## Skill picker (two-step domain then skill selection)
skill-domain = Domain
select-domain-prompt = Choose a domain…
skill-picker-help = Pick a domain to narrow the list of skills.
select-domain-first = Select a domain to choose a skill.
no-skills-in-domain = No skills in this domain yet.

## Requirements, Validations, Languages
add-requirement = Add requirement
required-level = Required level
validate-capability = Validate capability
validate-capability-help = As the central authority, set the validated level for this capability. The most recent validation sets the level directly and records you as the validating authority.
validate-button = Validate
validator-name = Validator's name
validated-level = Validated level
validated-by = Validated by { $name } on { $date }
not-yet-validated = Not yet validated
languages = Languages
language = Language
add-language = Add language
reading = Reading
writing = Writing
speaking = Speaking
not-specified = Not specified

## Tasks, Work, Publications
title = Title
tasks = Tasks
create-task = Create Task
edit-task = Edit Task
add-task = Add task
intended-outcome = Intended outcome
final-outcome = Final outcome
approval-tier = Approval tier
target-completion-date = Target completion date
completed-date = Completed date
status = Status
work-description = Work description
capability-level = Capability level
create-work = Add Work
edit-work = Edit Work
add-work = Add work
publications = Publications
new-publication-button = New Publication
create-publication = Create Publication
edit-publication = Edit Publication
publishing-organization = Publishing organization
lead-author = Lead author
subject = Subject
publishing-id = Publishing ID
published-date = Published date

## Index pages
roles = Roles
org-tiers = Organization Tiers
show-retired = Show retired
hide-retired = Hide retired

## Index search
search-placeholder = Search by name…
no-results = No matches.
showing-first = Showing first
refine-search = refine your search to narrow results.

restore-button = Restore

## Product
product = Product
products = Products
new-product-button = New Product
create-product = Create Product
edit-product = Edit Product
product-owner = Product owner
vacant-work = Vacant Work (unassigned)
explore = Explore
people = People
vacancies = Vacancies
analytics = Analytics

# Account onboarding & self-service (Phase 3)
create-person-account-note = Creating a person also creates a login-disabled account using the email below. The person cannot sign in until an administrator grants access.
my-profile = My profile
my-details = My details
account-status = Account
status-active = Active
status-invited = Invited
status-disabled = Disabled
status-provisioned = No access yet
grant-access = Grant access
activate-account = Activate account
activate-account-lead = Set a password to activate your account.
activate-missing-token = This activation link is missing its token. Please use the link your administrator sent you.
confirm-password = Confirm password
flag-an-issue = Flag an issue
flag-an-issue-help = See something wrong in your record? Send a note and an administrator will review it.
no-linked-person = Your account has no linked person record.

# Admin user portal
manage-users = Manage users
record-flags = Record flags
new-user = New user
edit-user = Edit user
no-users = No users found.
no-flags = No open flags. All clear.
user-role = Role
account-type = Account type
account-type-help = Agents are non-human service accounts and are not linked to a person.
password-blank-keep = Leave blank to keep the current password.
record-flags-help = Correction requests submitted by people about their own records.
message = Message
submitted = Submitted
view-record = View record
resolve = Resolve
enable-user = Enable
disable-user = Disable
actions = Actions
name = Name

# Assign work
assign-to-role = Assign to role
select-a-role = — Select a role —
select-an-organization = — Select an organization —
select-personnel-type = — Select a type —
select-an-org-tier = — Select an organization tier —
select-a-skill = — Select a skill —
select-a-task = — Select a task —
your-team = Your team
all-roles = All roles
