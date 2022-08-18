<script lang="ts">
  import {
    Header,
    HeaderNav,
    HeaderNavItem,
    HeaderNavMenu,
    SideNav,
    SideNavItems,
    SideNavMenu,
    SideNavMenuItem,
    SideNavLink,
    SideNavDivider,
    SkipToContent,
    Content,
    Grid,
    Row,
    Column,
    Button,
ToastNotification
  } from "carbon-components-svelte";
  // When using the Tauri API npm package:
  import { invoke } from '@tauri-apps/api/tauri';
  import {BaseDirectory } from "@tauri-apps/api/fs";

  let isSideNavOpen = false;

  async function invoke_thing() {
    try {
      console.log(await invoke("get_all_data_command", {root: "C:\\Users\\jdc10\\Desktop"}));
    } catch (error) {
      console.log(error);
    }
  }
</script>
<Header company="IBM" platformName="Carbon Svelte" bind:isSideNavOpen>
  <svelte:fragment slot="skip-to-content">
    <SkipToContent />
  </svelte:fragment>
  <HeaderNav>
    <HeaderNavItem href="/" text="Link 1" />
    <HeaderNavItem href="/" text="Link 2" />
    <HeaderNavItem href="/" text="Link 3" />
    <HeaderNavMenu text="Menu">
      <HeaderNavItem href="/" text="Link 1" />
      <HeaderNavItem href="/" text="Link 2" />
      <HeaderNavItem href="/" text="Link 3" />
    </HeaderNavMenu>
    <HeaderNavItem href="/" text="Link 4" />
  </HeaderNav>
</Header>

<SideNav bind:isOpen={isSideNavOpen}>
  <SideNavItems>
    <SideNavLink text="Formula" />
    <SideNavLink text="Other Thing" />
    <SideNavLink text="Link 3" />
    <SideNavMenu text="Menu">
      <SideNavMenuItem href="/" text="Link 1" />
      <SideNavMenuItem href="/" text="Link 2" />
      <SideNavMenuItem href="/" text="Link 3" />
    </SideNavMenu>
    <SideNavDivider />
    <SideNavLink text="Link 4" />
  </SideNavItems>
</SideNav>

<Content>
  <Grid>
    <Row>
      <Column>
        <h1>Welcome</h1>
        <Button on:click={invoke_thing}>
          Eeeee
        </Button>
      </Column>
    </Row>
  </Grid>
</Content>


