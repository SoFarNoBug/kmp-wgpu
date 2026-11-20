/**
 * Ohos / KBA composite for Holm `includeBuild`.
 *
 * Main `settings.gradle.kts` publishes Android/iOS/JS with stock Kotlin and has no
 * `ohosArm64`. Holm `settings.ohos` + `kmp.wgpu.local=true` must includeBuild **this**
 * directory so modules load `build.ohos.gradle.kts` (Kotlin 2.0.21-KBA-010).
 */
pluginManagement {
    repositories {
        google()
        gradlePluginPortal()
        mavenCentral()
        mavenLocal()
        maven { url = uri("https://mirrors.tencent.com/nexus/repository/maven-tencent/") }
    }
}

dependencyResolutionManagement {
    repositories {
        google()
        mavenCentral()
        mavenLocal()
        maven { url = uri("https://mirrors.tencent.com/nexus/repository/maven-tencent/") }
    }
}

rootProject.name = "kmp-wgpu-ohos"

include(":kmp-wgpu")
project(":kmp-wgpu").projectDir = file("../kmp-wgpu")
project(":kmp-wgpu").buildFileName = "build.ohos.gradle.kts"

include(":kmp-wgpu-surface")
project(":kmp-wgpu-surface").projectDir = file("../kmp-wgpu-surface")
project(":kmp-wgpu-surface").buildFileName = "build.ohos.gradle.kts"
