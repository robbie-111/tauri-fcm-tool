<script lang="ts">
  import { onMount } from "svelte"
  import { commands, type FcmConfig } from "$lib/bindings"

  let config = $state<FcmConfig>({
    oauthClientId: "",
    oauthRedirectUrl: "http://localhost:8080/callback",
    exchangeCodeUrl: "https://percent-config.111percent.net/ExchangeAuthorizationCode",
    firebaseProjectId: ""
  })

  let isAuthenticated = $state(false)
  let isLoading = $state(false)
  let message = $state("")
  let messageType = $state<"success" | "error" | "">("")

  onMount(async () => {
    await loadConfig()
    await checkAuth()
  })

  async function loadConfig() {
    try {
      const result = await commands.getConfig()
      if (result.status === "ok") {
        config = result.data
      } else {
        showMessage(`설정 로드 실패: ${result.error}`, "error")
      }
    } catch (e) {
      showMessage(`설정 로드 실패: ${e}`, "error")
    }
  }

  async function saveConfig() {
    isLoading = true
    try {
      const result = await commands.saveConfig(config)
      if (result.status === "ok") {
        showMessage("설정이 저장되었습니다", "success")
      } else {
        showMessage(`저장 실패: ${result.error}`, "error")
      }
    } catch (e) {
      showMessage(`저장 실패: ${e}`, "error")
    } finally {
      isLoading = false
    }
  }

  async function checkAuth() {
    try {
      const result = await commands.isAuthenticated()
      if (result.status === "ok") {
        isAuthenticated = result.data
      }
    } catch (e) {
      console.error("인증 상태 확인 실패:", e)
    }
  }

  async function handleLogin() {
    isLoading = true
    message = ""
    try {
      const result = await commands.startOauth()
      if (result.status === "ok") {
        if (result.data.success) {
          isAuthenticated = true
          showMessage("인증 성공!", "success")
        } else {
          showMessage(result.data.message, "error")
        }
      } else {
        showMessage(`인증 실패: ${result.error}`, "error")
      }
    } catch (e) {
      showMessage(`인증 실패: ${e}`, "error")
    } finally {
      isLoading = false
    }
  }

  async function handleLogout() {
    isLoading = true
    try {
      const result = await commands.logout()
      if (result.status === "ok") {
        isAuthenticated = false
        showMessage("로그아웃 되었습니다", "success")
      } else {
        showMessage(`로그아웃 실패: ${result.error}`, "error")
      }
    } catch (e) {
      showMessage(`로그아웃 실패: ${e}`, "error")
    } finally {
      isLoading = false
    }
  }

  function showMessage(msg: string, type: "success" | "error") {
    message = msg
    messageType = type
    setTimeout(() => {
      message = ""
      messageType = ""
    }, 5000)
  }
</script>

<div class="p-8 max-w-2xl mx-auto">
  <header class="mb-6">
    <h1 class="text-2xl font-bold mb-2">설정</h1>
    <p class="opacity-50">Firebase 및 OAuth 설정을 관리합니다</p>
  </header>

  {#if message}
    <div class="mb-4 p-3 rounded-lg {messageType === 'success' ? 'bg-success-500/20 text-success-500' : 'bg-error-500/20 text-error-500'}">
      {message}
    </div>
  {/if}

  <!-- Firebase 설정 -->
  <section class="card preset-filled-surface-200-800 p-6 mb-6">
    <h2 class="text-lg font-semibold mb-4">Firebase 설정</h2>
    
    <div class="space-y-4">
      <div>
        <label class="block text-sm font-medium mb-1" for="projectId">
          Firebase 프로젝트 ID <span class="text-error-500">*</span>
        </label>
        <input
          id="projectId"
          type="text"
          class="input w-full"
          placeholder="your-project-id"
          bind:value={config.firebaseProjectId}
        />
        <p class="text-xs opacity-50 mt-1">Firebase Console에서 확인할 수 있습니다</p>
      </div>
    </div>
  </section>

  <!-- OAuth 설정 -->
  <section class="card preset-filled-surface-200-800 p-6 mb-6">
    <h2 class="text-lg font-semibold mb-4">OAuth 2.0 설정</h2>
    
    <div class="space-y-4">
      <div>
        <label class="block text-sm font-medium mb-1" for="clientId">
          OAuth 클라이언트 ID <span class="text-error-500">*</span>
        </label>
        <input
          id="clientId"
          type="text"
          class="input w-full"
          placeholder="xxxxx.apps.googleusercontent.com"
          bind:value={config.oauthClientId}
        />
        <p class="text-xs opacity-50 mt-1">Google Cloud Console에서 "데스크톱 앱" 유형으로 생성</p>
      </div>

      <div>
        <label class="block text-sm font-medium mb-1" for="redirectUrl">
          리다이렉트 URL
        </label>
        <input
          id="redirectUrl"
          type="text"
          class="input w-full"
          bind:value={config.oauthRedirectUrl}
        />
      </div>

      <div>
        <label class="block text-sm font-medium mb-1" for="exchangeUrl">
          토큰 교환 API URL <span class="text-error-500">*</span>
        </label>
        <input
          id="exchangeUrl"
          type="text"
          class="input w-full"
          bind:value={config.exchangeCodeUrl}
        />
        <p class="text-xs opacity-50 mt-1">외부 서버에서 토큰 교환을 처리하는 API URL</p>
      </div>
    </div>
  </section>

  <!-- 저장 버튼 -->
  <div class="flex justify-end mb-8">
    <button
      class="btn preset-filled-primary-500"
      onclick={saveConfig}
      disabled={isLoading}
    >
      {isLoading ? "저장 중..." : "설정 저장"}
    </button>
  </div>

  <!-- 인증 상태 -->
  <section class="card preset-filled-surface-200-800 p-6">
    <h2 class="text-lg font-semibold mb-4">인증 상태</h2>
    
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <span class="w-3 h-3 rounded-full {isAuthenticated ? 'bg-success-500' : 'bg-error-500'}"></span>
        <span>{isAuthenticated ? "인증됨" : "인증 필요"}</span>
      </div>
      
      {#if isAuthenticated}
        <button
          class="btn preset-filled-error-500"
          onclick={handleLogout}
          disabled={isLoading}
        >
          로그아웃
        </button>
      {:else}
        <button
          class="btn preset-filled-primary-500"
          onclick={handleLogin}
          disabled={isLoading || !config.oauthClientId || !config.firebaseProjectId}
        >
          {isLoading ? "인증 중..." : "Google 계정으로 로그인"}
        </button>
      {/if}
    </div>

    {#if !isAuthenticated && (!config.oauthClientId || !config.firebaseProjectId)}
      <p class="text-xs text-warning-500 mt-3">
        로그인하려면 먼저 위의 필수 설정을 완료하고 저장해주세요
      </p>
    {/if}
  </section>
</div>
