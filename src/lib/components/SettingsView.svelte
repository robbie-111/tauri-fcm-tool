<script lang="ts">
  import { onMount } from "svelte"
  import { commands, type FcmConfig } from "$lib/bindings"

  let config = $state<FcmConfig>({
    oauthClientId: "",
    oauthRedirectUrl: "http://localhost:8080/callback",
    exchangeCodeUrl: "https://percent-config.111percent.net/ExchangeAuthorizationCode",
    firebaseProjectId: ""
  })

  // Firebase 프로젝트 설정 분리
  let environment = $state<"dev" | "qa">("dev")
  let projectName = $state("")

  // 테마 설정
  let theme = $state<"system" | "light" | "dark">("system")

  let isAuthenticated = $state(false)
  let isLoading = $state(false)
  let message = $state("")
  let messageType = $state<"success" | "error" | "">("")

  onMount(async () => {
    await loadConfig()
    await checkAuth()
    loadTheme()
  })

  async function loadConfig() {
    try {
      const result = await commands.getConfig()
      if (result.status === "ok") {
        config = result.data
        // firebaseProjectId에서 환경과 프로젝트명 분리
        if (config.firebaseProjectId.startsWith("dev-")) {
          environment = "dev"
          projectName = config.firebaseProjectId.slice(4)
        } else if (config.firebaseProjectId.startsWith("qa-")) {
          environment = "qa"
          projectName = config.firebaseProjectId.slice(3)
        } else if (config.firebaseProjectId) {
          // 기존 형식이 아닌 경우 그대로 프로젝트명으로 사용
          projectName = config.firebaseProjectId
        }
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
      // 환경과 프로젝트명을 조합하여 firebaseProjectId 설정
      config.firebaseProjectId = `${environment}-${projectName}`
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

  function loadTheme() {
    const savedTheme = localStorage.getItem("theme") as "system" | "light" | "dark" | null
    if (savedTheme) {
      theme = savedTheme
    }
  }

  function applyTheme(selectedTheme: "system" | "light" | "dark") {
    const html = document.documentElement
    if (selectedTheme === "system") {
      const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches
      html.classList.toggle("dark", prefersDark)
    } else {
      html.classList.toggle("dark", selectedTheme === "dark")
    }
    localStorage.setItem("theme", selectedTheme)
  }

  function handleThemeChange() {
    applyTheme(theme)
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

  <!-- 테마 설정 -->
  <div class="flex items-center gap-2 mb-6">
    <label for="theme" class="text-sm opacity-70">Theme:</label>
    <select 
      id="theme" 
      class="select select-sm preset-outlined-surface-500 w-28" 
      bind:value={theme} 
      onchange={handleThemeChange}
    >
      <option value="system">System</option>
      <option value="dark">Dark</option>
      <option value="light">Light</option>
    </select>
  </div>

  <!-- Firebase 설정 -->
  <section class="card preset-filled-surface-200-800 p-6 mb-6">
    <h2 class="text-lg font-semibold mb-4">Firebase 설정</h2>
    
    <div class="space-y-4">
      <div>
        <label class="block text-sm font-medium mb-1" for="projectId">
          Firebase 프로젝트 ID <span class="text-error-500">*</span>
        </label>
        <div class="flex gap-2">
          <select class="select w-24" bind:value={environment}>
            <option value="dev">dev</option>
            <option value="qa">qa</option>
          </select>
          <input
            id="projectId"
            type="text"
            class="input flex-1"
            placeholder="프로젝트명 (예: gamebase)"
            bind:value={projectName}
          />
        </div>
        <p class="text-xs opacity-50 mt-1">환경 접두사와 프로젝트명이 조합되어 저장됩니다 (예: {environment}-{projectName || "gamebase"})</p>
      </div>
    </div>
  </section>

  <!-- 저장 버튼 -->
  <div class="flex justify-end mb-6">
    <button
      class="btn preset-filled-primary-500"
      onclick={saveConfig}
      disabled={isLoading}
    >
      {isLoading ? "저장 중..." : "설정 저장"}
    </button>
  </div>

  <!-- 인증 상태 -->
  <section class="card preset-filled-surface-200-800 p-6 mb-6">
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
          disabled={isLoading || !config.oauthClientId || !projectName}
        >
          {isLoading ? "인증 중..." : "Google 계정으로 로그인"}
        </button>
      {/if}
    </div>

    {#if !isAuthenticated && (!config.oauthClientId || !projectName)}
      <p class="text-xs text-warning-500 mt-3">
        로그인하려면 먼저 위의 필수 설정을 완료하고 저장해주세요
      </p>
    {/if}
  </section>

  <!-- OAuth 설정 -->
  <section class="card preset-filled-surface-200-800 p-6">
    <h2 class="text-lg font-semibold mb-4">OAuth 2.0 설정</h2>
    
    <div class="space-y-4">
      <div>
        <label class="block text-sm font-medium mb-1 opacity-60" for="clientId">
          OAuth 클라이언트 ID
        </label>
        <input
          id="clientId"
          type="text"
          class="input w-full opacity-60 cursor-not-allowed"
          value={config.oauthClientId}
          disabled
        />
      </div>

      <div>
        <label class="block text-sm font-medium mb-1 opacity-60" for="redirectUrl">
          리다이렉트 URL
        </label>
        <input
          id="redirectUrl"
          type="text"
          class="input w-full opacity-60 cursor-not-allowed"
          value={config.oauthRedirectUrl}
          disabled
        />
      </div>

      <div>
        <label class="block text-sm font-medium mb-1 opacity-60" for="exchangeUrl">
          토큰 교환 API URL
        </label>
        <input
          id="exchangeUrl"
          type="text"
          class="input w-full opacity-60 cursor-not-allowed"
          value={config.exchangeCodeUrl}
          disabled
        />
      </div>
    </div>
  </section>
</div>
