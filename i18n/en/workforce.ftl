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
org-chart-explore-help = Click the + on a tier to expand it, or on a team to load its roles and people. Box colour shows team capacity.
how-to-explore = How to explore
visual-view = Visual view
list-view = List view
people-label = people
effort-label = effort
tier-label = Tier
expand = Expand
collapse = Collapse
show-members = Show roles and people
leadership-team = Leadership team
working-teams = Working teams

# Manager panel / transfer offers
manage-title = Manager panel
manage-help = Review transfer offers for your team. Accept an incoming offer to move the person onto the offering team; decline to keep them.
manage-incoming = Incoming offers (awaiting your decision)
manage-outgoing = Offers you have made
manage-no-incoming = No offers awaiting your decision.
manage-no-outgoing = You have not made any offers.
offer-to = To
offer-from = From
offer-offered-by = Offered by
offer-approver = Approver
offer-note-label = Optional note
offer-accept = Accept
offer-decline = Decline
offer-withdraw = Withdraw

# Activity log (admin)
activity-title = Activity log
activity-help = Recent changes recorded across the system.
activity-empty = No activity recorded yet.
activity-when = When
activity-action = Action
activity-entity = Entity
activity-summary = Summary
activity-actor = By
activity-system = System
zoom-in = Zoom in
zoom-out = Zoom out
zoom-reset = Reset zoom
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
role-edit-limited = Titles, the active flag and dates can be changed here. To change the classification or the person, end this role and create a new one, or use Assign/Vacate — that preserves the history.
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
created-by-role = Created by role
unassigned = Unassigned
previous = Previous
next = Next
page = Page
pagination = Pagination

# Work tracking (Tier 1: dates + blocked context)
due-date = Due date
overdue = Overdue
started = Started
completed = Completed
blocked-label = Blocked
blocked-since = since
blocked-reason = Why is this blocked?
blocked-on-role = Waiting on (role)
blocked-on-none = — Not waiting on a specific role —
blocked-help = This work is blocked. Note why, and optionally the role you are waiting on, so it can be followed up.
waiting-on = Waiting on
vacant-position = vacant position
status-history = Status history
no-status-history = No status changes recorded yet.
my-work = My Work
my-work-role = Role
my-work-total = Assigned to me
my-work-empty = You have no work assigned. Nice.
my-work-no-person = Your account isn't linked to a person record, so there's no personal worklist to show.
updates-and-flags = Updates & Flags
open-flags = open
add-update = Add a comment or flag
update-placeholder = Share an update, or flag a blocker for management attention…
kind-comment = Comment
kind-flag = Flag for attention
post-update = Post
flag-resolved = Flag resolved
resolve-flag = Resolve flag
resolved-by = Resolved by
no-updates = No updates yet.
flags-queue = Flags
flag = Flag
flags-queue-help = Open "needs attention" flags on work you manage. Resolve them here or open the work item for full context.
flags-queue-empty = No open flags. All clear.
view = View
approval = Approval
approvals = Approvals
approved-by = Approved by
rejected-by = Rejected by
rejection-reason = Rejection reason
submit-for-approval = Submit for approval
approve = Approve
reject = Reject
pending = pending
approvals-help = Tasks awaiting your approval. Approve or reject them here; a rejection needs a reason.
approvals-empty = Nothing awaiting approval.
dependencies = Dependencies
cant-start = Can't start yet
blocked-by = Blocked by
no-dependencies = Not blocked by anything.
select-dependency = — Select a work item —
add-dependency = Add
dependencies-help = A dependency means this work can't start until the other item is completed. Only sibling work under this task is offered.
blocks-label = Blocks
remove = Remove

# Priority consistency (Proposal 7c)
priority-consistency = Priority Consistency
priority-consistency-tagline = Where priority drops between a product, its tasks, and their work
priority-mismatches = Priority Mismatches
flagged-tasks = Flagged Tasks
below-product-label = Below its product
below-work-label = Work below their task
below-work-short = below task
task-priority = Task Priority
product-priority = Product Priority
issue = Issue
no-priority-mismatches = All priorities are consistent across products, tasks, and work.
priority-below-product = Priority below product
priority-below-task = Priority below task
lower-priority-work = lower-priority items

# Analytics — shared
workforce-analytics = Workforce Analytics
capability-coverage = Capability Coverage
delivery-map = Delivery Map
talent-mobility = Talent Mobility
capability-growth = Capability Growth
supply-vs-demand = Supply vs Demand
team-capacity = Team Capacity
capability-gaps = Capability Gaps
no-data-available = No data available.
count-label = Count
domain-label = Domain
gap-label = Gap

# Analytics — dashboard sections
work-by-status = Work by Status
total-suffix = total
unassigned-suffix = unassigned
bar-label = Bar
no-work-data = No work data available.
work-effort-by-domain = Work Effort by Domain
total-effort = Total Effort
total-active-effort-hint = (total active effort)
active-suffix = active
load-label = Load
load-high = High
load-medium = Medium
load-low = Low
no-team-data = No team data available.
over-allocated-people = Over-Allocated People
over-allocated-hint = (effort ≥ 8)
available-suffix = available
no-one-over-allocated = No one is over-allocated.
vacant-roles = Vacant Roles
no-vacant-roles = No vacant roles.
capability-gap-by-domain = Capability Gap by Domain
required-label = Required
available-label = Available
gap-footnote = Required = role requirements. Available = validated capabilities (or self-identified where unvalidated). A negative gap (red) means fewer qualified people than required positions.

# Analytics — capability coverage
team-capability-coverage = Team Capability Coverage
coverage-tagline = Depth score = sum of capability level weights per team × domain
active-teams = Active Teams
domains-with-coverage = Domains with Coverage
peak-depth-score = Peak Depth Score
capability-heatmap = Capability Heatmap (Teams × Domains)
heatmap-footnote = Cell value = sum of level weights across all active people in each team (Desired=1, Novice=2, Experienced=3, Expert=4, Specialist=5). Validated level is preferred; self-identified is used where unvalidated.
domain-strength-ranking = Domain Strength Ranking
depth-label = Depth
coverage-table = Coverage Table
no-team-capability-data = No team capability data available.

# Analytics — delivery map
delivery-tagline = Product → Task → Work, sized by effort
work-items = Work Items
delivery-treemap = Delivery Treemap
selected-label = Selected:
no-products-with-tasks = No products with tasks to display.
treemap-footnote = Rectangle area = work effort. Leaf colour = work status. Click a product or task to zoom in (use the breadcrumb to zoom back out); a link to open that product, task or work item appears above the chart.
products-by-effort = Products by Effort
no-products-available = No products available.

# Analytics — capability growth
growth-tagline = Cumulative validated capability depth over time by domain
active-domains = Active Domains
current-total-depth = Current Total Depth
capability-depth-over-time = Capability Depth Over Time (Quarterly)
growth-footnote = Each line represents a skill domain. Value = cumulative weighted depth of validated capabilities (Desired=1, Novice=2, Experienced=3, Expert=4, Specialist=5).

# Analytics — talent mobility
mobility-tagline = Previous org tier → current org tier movement
total-moves = Total Moves
promotions = Promotions
lateral-moves = Lateral Moves
inflows = Inflows
outflows = Outflows
org-tiers-involved = Org Tiers Involved
movement-flow = Movement Flow
no-moves-found = No cross-org-tier moves found. Movement is detected when a person's most recent prior role was in a different team than their current role.
mobility-footnote = Each flow links a person's previous org tier (was) to their current org tier (now), sized by the number of people who made that move (moves within the same org tier are not shown). Hover a flow to highlight it.
transitions = Transitions
from-previous-org-tier = From (previous org tier)
to-current-org-tier = To (current org tier)

# Analytics — supply vs demand
capability-supply-vs-demand = Capability Supply vs Demand
supply-demand-tagline = Per-domain capability availability vs role/work requirements
domains-tracked = Domains Tracked
domains-with-surplus = Domains with Surplus
domains-with-deficit = Domains with Deficit
supply-label = Supply:
demand-label = Demand:
no-supply-demand-data = No supply/demand data available.

# Entity detail pages — shared headings
contact-info = Contact Info
address-label = Address
details-heading = Details
past-roles = Past Roles
potential-job-matches = Potential Job Matches
assign-to-this-role = Assign to this role
no-work-assigned = No work assigned.
current-effort = Current Effort
team-owner = Team Owner
vacant-label = Vacant
capabilities-heading = Capabilities
assigned-to = Assigned To
not-assigned = Not assigned.
reassign-button = Reassign
assign-button = Assign
meets-required-level = Meets required level
below-required-level = Below required level
qualified-people = Qualified People
qualified-people-hint = Validated at or above { $level } in { $skill }
no-qualified-people = No qualified people found for this work's requirements.
currently-assigned = Currently assigned
assign-as = Assign as
no-active-role-hint = No active role — assign a role first
website-label = Website
domain-level-label = Domain → Level
priority-label = Priority
required-skill = Required Skill
target-completion = Target Completion
assigned-by = Assigned by

# Coverage view toggle (tier rollup vs per-team)
view-by-tier2 = Tier 2 view
view-by-team = Team view
active-org-tiers = Tier-2 Org Tiers
coverage-rollup-hint = Aggregated at org-tier level 2; a person on several teams under one tier counts once.

# Org chart fullscreen toggle
fullscreen = Full screen
exit-fullscreen = Exit full screen

# Shared destructive-action confirmation
please-confirm = Please confirm
vacate-role = Vacate role

# Role classification chooser
classification-prompt = Classification
classification-none = Unclassified
military-label = Military
civilian-label = Civilian
no-requirements-specified = No requirements specified.
assign-person-direct-help = Start typing a name and pick from the suggestions, or use the candidate matcher below.

# Person page self-service band
this-is-you = This is your record.
update-my-details = Update my details
add-my-capability = Add a capability
add-my-capability-help = Declare a skill you hold and your own assessment of your level. An administrator can validate it later; validated levels always take precedence.

# Role reporting line & team vacancies
reporting-line = Reporting line
reports-to = Reports to
direct-reports = Direct reports
no-manager = None — top position for its team
find-candidates = Find candidates
due-short = Due

# Role page — remaining headings & matcher panel
requirements-heading = Requirements
potential-matches = Potential Matches
assignment-history = Assignment History
incumbent = Incumbent
current-badge = Current
incumbent-fit = Incumbent fit — required vs held
incumbent-fit-help = Green bars meet or exceed the required level; red bars fall short. Levels: Desired (1) → Specialist (5).
min-match-label = Minimum match
min-match-help = How much of the requirement set a close candidate must meet.
max-gap-label = Allowed skill gap
levels-suffix = level(s)
max-gap-help = Largest shortfall tolerated on any single skill.
add-requirements-hint = Add requirements to this role to find matching candidates.
could-not-load-matches = Could not load matches
in-your-area = In your area
in-your-area-hint = Under the org tier this role's owner manages — reassign directly.
full-matches = Full matches
full-matches-hint = Meet every requirement at or above the required level.
close-matches = Close matches
close-matches-hint = At least { $pct }% of requirements met, with no single skill short by more than { $gap } level(s).
no-full-in-area = No one in your area meets every requirement.
no-close-in-area = No close matches in your area at this threshold.
elsewhere-in-org = Elsewhere in the organization
elsewhere-hint = Outside your area — contact the person's manager to arrange a move.
no-full-elsewhere = No full matches elsewhere.
no-close-elsewhere = No close matches elsewhere at this threshold.
meets-all-reqs = 100% — meets all { $total } requirements
match-pct = { $pct }% match
reqs-met = { $met }/{ $total } requirements met
match-score-label = Match score
meets-requirement = Meets requirement
short-by = Short by { $gap } level(s)
needs-label = needs
has-label = has
none-held = none held
current-role-label = Current role
managed-by = Managed by
make-offer = Make offer
offer-message-placeholder = Optional message to their manager

# Team page — remaining headings
headcount = Headcount
team-members = Team Members
filled-suffix = filled
vacant-suffix = vacant
no-members-assigned = No members assigned.
delivery-at-a-glance = Delivery at a Glance
active-work-heading = Active Work
no-products-linked = No products linked to this team's work.
no-tasks-underway = No tasks underway.
no-work-in-progress = No work currently in progress.
capabilities-by-domain = Capabilities by Domain
total-capabilities = Total Capabilities

# Financial module
contracts-heading = Contracts
add-contract = Add contract
edit-contract = Edit contract
contract-reference = Contract reference number
contract-vendor = Vendor
contract-period = Period
contract-value = Total value ($)
contract-value-help = Full contract value in dollars, recognized evenly across the period.
contract-fy-share = Fiscal-year share
no-contracts = No contracts recorded under this task.
delete-button = Delete
level = Level
pay-rates = Pay rates
pay-rates-intro = Annual salary for each classification. A role is priced by the rate in force for its group and level (or rank); superseding a rate means adding a newer effective date, so history is preserved.
civilian-rates = Civilian classifications
military-rates = Military ranks
add-pay-rate = Add a pay rate
add-pay-rate-help = Fill in either an occupational group and level, or a rank.
annual-rate = Annual rate ($)
annual-rate-help = Annual salary in dollars for this classification.
effective-date = Effective date
cost-heading = Cost
annual-salary = Annual salary
fy-budget = Budget
fy-projected = Projected to Mar 31
fy-lapse = Vacancy lapse
