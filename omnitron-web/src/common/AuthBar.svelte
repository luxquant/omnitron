<script lang="ts">
    import { faSignOut } from "@fortawesome/free-solid-svg-icons";
    import Fa from "svelte-fa";

    import { api } from "gateway/lib/api";
    import { serverInfo, reloadServerInfo } from "gateway/lib/store";
    import {
        Button,
        Dropdown,
        DropdownItem,
        DropdownMenu,
        DropdownToggle,
    } from "@sveltestrap/sveltestrap";

    async function logout() {
        await api.logout();
        await reloadServerInfo();
        location.href = "/@omnitron";
    }
</script>

{#if $serverInfo?.username}
    <div class="ms-auto">
        <a href="/#/profile">
            {$serverInfo.username}
        </a>
        {#if $serverInfo.authorizedViaTicket}
            <span class="ml-2">(ticket auth)</span>
        {/if}
    </div>

    <Button color="link" on:click={logout} title="Log out" class="p-0 ms-2">
        <Fa icon={faSignOut} fw />
    </Button>
{/if}
