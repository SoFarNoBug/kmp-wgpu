plugins {
    kotlin("multiplatform")
    id("com.android.library")
}

kotlin {
    androidTarget {
        compilerOptions {
            jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_1_8)
        }
    }

    ohosArm64()

    sourceSets {
        val commonMain by getting {
            dependencies {
                api(project(":kmp-wgpu"))
            }
        }
        val androidMain by getting
        val ohosArm64Main by getting
    }
}

android {
    namespace = "com.jlj.kmp.wgpu.surface"
    compileSdk = 34
    buildToolsVersion = "34.0.0"
    defaultConfig {
        minSdk = 21
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }
}
