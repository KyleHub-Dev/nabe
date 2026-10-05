# Nabe households

Language for installing and managing DNS filtering in a household.

## Language

**Household**:
The people sharing a home network and its filtering settings. A household is
separate from the person operating Nabe for several homes.
_Avoid_: Customer, organization

**Household administrator**:
A person allowed to manage their household's filtering and appliance.
_Avoid_: Platform administrator

**Operator**:
The person responsible for the shared Nabe installation and supporting its
households.
_Avoid_: Household owner, superuser

**Appliance**:
The local machine serving a household's DNS. The appliance is distinct from the
phones and computers using it.
_Avoid_: Client device

**Device Client**:
A phone, computer or other device whose DNS settings and activity Nabe can
identify and manage.
_Avoid_: Appliance, user

**Pairing**:
The setup step that associates a particular appliance with the intended
household.
_Avoid_: Login

**Recovery access**:
Local access used to diagnose or recover an appliance when normal dashboard
management is unavailable.
_Avoid_: Remote shell, second dashboard
