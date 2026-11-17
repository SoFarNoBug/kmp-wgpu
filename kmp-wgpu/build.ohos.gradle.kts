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

    ohosArm64 {
        compilations.getByName("main") {
            cinterops {
                create("kmp_wgpu_native") {
                    defFile(project.file("src/nativeInterop/cinterop/kmp_wgpu_native.def"))
                    includeDirs(project.file("../kmp-wgpu-native/include"))
                }
            }
        }
    }

    sourceSets {
        val commonMain by getting
        val androidMain by getting
        val ohosArm64Main by getting
    }
}

android {
    namespace = "com.jlj.kmp.wgpu"
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
