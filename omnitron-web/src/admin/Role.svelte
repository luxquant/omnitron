<script lang="ts">
    import { api, type Role } from 'admin/lib/api'
    import AsyncButton from 'common/AsyncButton.svelte'
    import { replace } from 'svelte-spa-router'
    import { FormGroup } from '@sveltestrap/sveltestrap'
    import { stringifyError } from 'common/errors'
    import Alert from 'common/sveltestrap-s5-ports/Alert.svelte'
    import Loadable from 'common/Loadable.svelte'

    interface Props {
        params: { id: string };
    }

    let { params }: Props = $props()

    let error: string|null = $state(null)
    let role: Role | undefined = $state()
    const initPromise = init()

    async function init () {
        role = await api.getRole({ id: params.id })
    }

    async function update () {
        try {
            role = await api.updateRole({
                id: params.id,
                roleDataRequest: role!,
            })
        } catch (err) {
            error = await stringifyError(err)
        }
    }

    async function remove () {
        if (confirm(`Delete role ${role!.name}?`)) {
            await api.deleteRole(role!)
            replace('/config/roles')
        }
    }
</script>

<Loadable promise={initPromise}>
    <div class="page-summary-bar">
        <div>
            <h1>{role!.name}</h1>
            <div class="text-muted">role</div>
        </div>
    </div>

    <FormGroup floating label="Name">
        <input class="form-control" bind:value={role!.name} />
    </FormGroup>
</Loadable>

{#if error}
    <Alert color="danger">{error}</Alert>
{/if}

<div class="d-flex">
    <AsyncButton
    color="primary"
        class="ms-auto"
        click={update}
    >Update</AsyncButton>

    <AsyncButton
        class="ms-2"
        color="danger"
        click={remove}
    >Remove</AsyncButton>
</div>
