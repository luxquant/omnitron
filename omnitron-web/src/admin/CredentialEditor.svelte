<script lang="ts" module>
    export type ExistingCredential =
        { kind: typeof CredentialKind.Password } & ExistingPasswordCredential
        | { kind: typeof CredentialKind.PublicKey } & ExistingPublicKeyCredential
        | { kind: typeof CredentialKind.Totp } & ExistingOtpCredential
</script>

<script lang="ts">
    import { faIdBadge, faKey, faKeyboard, faMobileScreen } from '@fortawesome/free-solid-svg-icons'
    import { api, CredentialKind, type ExistingPasswordCredential, type ExistingPublicKeyCredential, type ExistingOtpCredential, type UserRequireCredentialsPolicy } from 'admin/lib/api'
    import Fa from 'svelte-fa'
    import { Button } from '@sveltestrap/sveltestrap'
    import CreatePasswordModal from './CreatePasswordModal.svelte'
    import SsoCredentialModal from './SsoCredentialModal.svelte'
    import PublicKeyCredentialModal from './PublicKeyCredentialModal.svelte'
    import CreateOtpModal from './CreateOtpModal.svelte'
    import AuthPolicyEditor from './AuthPolicyEditor.svelte'
    import { possibleCredentials } from 'common/protocols'
    import CredentialUsedStateBadge from 'common/CredentialUsedStateBadge.svelte'
    import Loadable from 'common/Loadable.svelte'

    interface Props {
        userId: string
        username: string
        credentialPolicy: UserRequireCredentialsPolicy,
    }
    let { userId, username, credentialPolicy = $bindable() }: Props = $props()

    let credentials: ExistingCredential[] = $state([])

    let creatingPassword = $state(false)
    let creatingOtp = $state(false)
    let editingPublicKeyCredential = $state(false)
    let editingPublicKeyCredentialInstance: ExistingPublicKeyCredential|null = $state(null)

    const loadPromise = load()

    const policyProtocols: { id: 'ssh' | 'http' | 'mysql' | 'postgres', name: string }[] = [
        { id: 'ssh', name: 'SSH' },
        { id: 'http', name: 'HTTP' },
        { id: 'mysql', name: 'MySQL' },
        { id: 'postgres', name: 'PostgreSQL' },
    ]

    async function load () {
        await Promise.all([
            loadPasswords(),
            loadPublicKeys(),
            loadOtp(),
        ])
    }

    async function loadPasswords () {
        credentials.push(...(await api.getPasswordCredentials({ userId })).map(c => ({
            kind: CredentialKind.Password,
            ...c,
        })))
    }

    async function loadPublicKeys () {
        credentials.push(...(await api.getPublicKeyCredentials({ userId })).map(c => ({
            kind: CredentialKind.PublicKey,
            ...c,
        })))
    }

    async function loadOtp () {
        credentials.push(...(await api.getOtpCredentials({ userId })).map(c => ({
            kind: CredentialKind.Totp,
            ...c,
        })))
    }

    async function deleteCredential (credential: ExistingCredential) {
        credentials = credentials.filter(c => c !== credential)
        if (credential.kind === CredentialKind.Password) {
            await api.deletePasswordCredential({
                id: credential.id,
                userId,
            })
        }
        if (credential.kind === CredentialKind.PublicKey) {
            await api.deletePublicKeyCredential({
                id: credential.id,
                userId,
            })
        }
        if (credential.kind === CredentialKind.Totp) {
            await api.deleteOtpCredential({
                id: credential.id,
                userId,
            })
        }
    }

    async function createPassword (password: string) {
        const credential = await api.createPasswordCredential({
            userId,
            newPasswordCredential: {
                password,
            },
        })
        credentials.push({
            kind: CredentialKind.Password,
            ...credential,
        })
    }

    async function createOtp (secretKey: number[]) {
        const credential = await api.createOtpCredential({
            userId,
            newOtpCredential: {
                secretKey,
            },
        })
        credentials.push({
            kind: CredentialKind.Totp,
            ...credential,
        })

        // Automatically set up a 2FA policy when adding an OTP
        for (const protocol of ['http', 'ssh'] as ('http'|'ssh')[]) {
            for (const ck of [CredentialKind.Password, CredentialKind.PublicKey]) {
                if (
                    !credentialPolicy[protocol]
                    && credentials.some(x => x.kind === ck)
                    && possibleCredentials[protocol]?.has(ck)
                ) {
                    credentialPolicy = {
                        ...credentialPolicy ?? {},
                        [protocol]: [ck, CredentialKind.Totp],
                    }
                }
            }
        }
    }

    async function savePublicKeyCredential (label: string, opensshPublicKey: string) {
        if (editingPublicKeyCredentialInstance) {
            editingPublicKeyCredentialInstance.label = label
            editingPublicKeyCredentialInstance.opensshPublicKey = opensshPublicKey
            await api.updatePublicKeyCredential({
                userId,
                id: editingPublicKeyCredentialInstance.id,
                newPublicKeyCredential: editingPublicKeyCredentialInstance,
            })
        } else {
            const credential = await api.createPublicKeyCredential({
                userId,
                newPublicKeyCredential: {
                    label,
                    opensshPublicKey,
                },
            })
            credentials.push({
                kind: CredentialKind.PublicKey,
                ...credential,
            })
        }
        editingPublicKeyCredential = false
        editingPublicKeyCredentialInstance = null
    }

    function abbreviatePublicKey (key: string) {
        return key.slice(0, 16) + '...' + key.slice(-8)
    }

    function assertDefined<T>(value: T|undefined): T {
        if (value === undefined) {
            throw new Error('Value is undefined')
        }
        return value
    }
</script>

<div class="d-flex align-items-center mt-4 mb-2">
    <h4 class="m-0">Credentials</h4>
    <span class="ms-auto"></span>
    <Button size="sm" color="link" on:click={() => creatingPassword = true}>
        Add password
    </Button>
    <Button size="sm" color="link" on:click={() => {
        editingPublicKeyCredentialInstance = null
        editingPublicKeyCredential = true
    }}>Add public key</Button>
    <Button size="sm" color="link" on:click={() => creatingOtp = true}>Add OTP</Button>
</div>

<Loadable promise={loadPromise}>
    <div class="list-group list-group-flush mb-3">
        {#each credentials as credential}
        <div class="list-group-item credential">
            {#if credential.kind === CredentialKind.Password }
                <Fa fw icon={faKeyboard} />
                <span class="label me-auto">Password</span>
            {/if}
            {#if credential.kind === 'PublicKey'}
                <Fa fw icon={faKey} />
                <div class="main me-auto">
                    <div class="label d-flex align-items-center">
                        {credential.label}
                    </div>
                    <small class="d-block text-muted">{abbreviatePublicKey(credential.opensshPublicKey)}</small>
                </div>
                <CredentialUsedStateBadge credential={credential} />
                <div class="me-2"></div>
            {/if}
            {#if credential.kind === 'Totp'}
                <Fa fw icon={faMobileScreen} />
                <span class="label me-auto">One-time password</span>
            {/if}

            {#if credential.kind === CredentialKind.PublicKey}
            <a
                class="ms-2"
                href={''}
                onclick={e => {
                    editingPublicKeyCredentialInstance = credential
                    editingPublicKeyCredential = true
                    e.preventDefault()
                }}>
                Change
            </a>
            {/if}
            <a
                class="ms-2"
                href={''}
                onclick={e => {
                    deleteCredential(credential)
                    e.preventDefault()
                }}>
                Delete
            </a>
        </div>
        {/each}
    </div>

    <h4>Auth policy</h4>
    <div class="list-group list-group-flush mb-3">
        {#each policyProtocols as protocol}
        <div class="list-group-item">
            <div>
                <strong>{protocol.name}</strong>
            </div>
            {#if possibleCredentials[protocol.id]}
                {@const _possibleCredentials = assertDefined(possibleCredentials[protocol.id])}
                <AuthPolicyEditor
                    bind:value={credentialPolicy}
                    existingCredentials={credentials}
                    possibleCredentials={_possibleCredentials}
                    protocolId={protocol.id}
                />
            {/if}
        </div>
        {/each}
    </div>
</Loadable>

{#if creatingPassword}
<CreatePasswordModal
    bind:isOpen={creatingPassword}
    create={createPassword}
/>
{/if}

{#if creatingOtp}
<CreateOtpModal
    bind:isOpen={creatingOtp}
    {username}
    create={createOtp}
/>
{/if}

{#if editingPublicKeyCredential}
<PublicKeyCredentialModal
    bind:isOpen={editingPublicKeyCredential}
    instance={editingPublicKeyCredentialInstance ?? undefined}
    save={savePublicKeyCredential}
/>
{/if}

<style lang="scss">
    .credential {
        display: flex;
        align-items: center;

        .label:not(:first-child), .main {
            margin-left: .75rem;
        }
    }
</style>
