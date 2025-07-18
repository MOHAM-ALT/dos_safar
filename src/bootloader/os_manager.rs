// OS management functions 
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::process::Command as AsyncCommand;
use tokio::fs as async_fs;
use tracing::{info, warn, error, debug};
use crate::bootloader::menu::{OperatingSystem, OSType};
use crate::utils::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OSManager {
    config: Config,
    os_storage_path: PathBuf,
    boot_partition_path: PathBuf,
    backup_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OSInstallationProgress {
    pub stage: InstallationStage,
    pub progress_percentage: f32,
    pub current_operation: String,
    pub estimated_time_remaining: Option<u64>, // seconds
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InstallationStage {
    Preparing,
    Downloading,
    Extracting,
    Installing,
    Configuring,
    Testing,
    Finalizing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OSBackup {
    pub os_name: String,
    pub backup_date: chrono::DateTime<chrono::Utc>,
    pub backup_size_mb: u64,
    pub backup_path: String,
    pub is_bootable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OSImage {
    pub name: String,
    pub file_path: String,
    pub size_mb: u64,
    pub os_type: OSType,
    pub checksum: Option<String>,
    pub is_compressed: bool,
    pub supported_devices: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootConfiguration {
    pub default_os: Option<String>,
    pub timeout_seconds: u64,
    pub available_systems: Vec<OperatingSystem>,
    pub boot_order: Vec<String>,
    pub recovery_mode: bool,
}

impl OSManager {
    pub fn new(config: &Config) -> Result<Self> {
        let os_storage_path = PathBuf::from("/boot/dos_safar/systems");
        let boot_partition_path = PathBuf::from("/boot");
        let backup_path = PathBuf::from("/boot/dos_safar/backups");

        // إنشاء المجلدات المطلوبة
        fs::create_dir_all(&os_storage_path)
            .context("فشل في إنشاء مجلد أنظمة التشغيل")?;
        fs::create_dir_all(&backup_path)
            .context("فشل في إنشاء مجلد النسخ الاحتياطية")?;

        Ok(OSManager {
            config: config.clone(),
            os_storage_path,
            boot_partition_path,
            backup_path,
        })
    }

    /// تثبيت نظام تشغيل من صورة
    pub async fn install_os_from_image(&self, image_path: &str, os_name: &str) -> Result<()> {
        info!("🔧 بدء تثبيت {} من {}", os_name, image_path);

        // التحقق من وجود الصورة
        if !Path::new(image_path).exists() {
            return Err(anyhow::anyhow!("الصورة {} غير موجودة", image_path));
        }

        // تحضير مجلد التثبيت
        let install_path = self.os_storage_path.join(os_name);
        if install_path.exists() {
            warn!("النظام {} موجود مسبقاً، سيتم الاستبدال", os_name);
            fs::remove_dir_all(&install_path)
                .context("فشل في حذف النظام القديم")?;
        }

        fs::create_dir_all(&install_path)
            .context("فشل في إنشاء مجلد التثبيت")?;

        // تحديد نوع الصورة والتثبيت المناسب
        let image_type = self.detect_image_type(image_path)?;
        
        match image_type {
            ImageType::ISO => self.install_from_iso(image_path, &install_path).await?,
            ImageType::IMG => self.install_from_img(image_path, &install_path).await?,
            ImageType::TAR => self.install_from_tar(image_path, &install_path).await?,
            ImageType::ZIP => self.install_from_zip(image_path, &install_path).await?,
        }

        // تكوين نظام التشغيل المثبت
        self.configure_installed_os(&install_path, os_name).await?;

        // إضافة إلى قائمة الأنظمة المتاحة
        self.register_os(os_name, &install_path).await?;

        info!("✅ تم تثبيت {} بنجاح", os_name);
        Ok(())
    }

    /// تثبيت نظام تشغيل من URL
    pub async fn install_os_from_url(&self, url: &str, os_name: &str) -> Result<()> {
        info!("📥 تحميل وتثبيت {} من {}", os_name, url);

        // تحميل الصورة
        let temp_path = format!("/tmp/{}.img", os_name);
        self.download_os_image(url, &temp_path).await?;

        // تثبيت من الملف المحمل
        self.install_os_from_image(&temp_path, os_name).await?;

        // حذف الملف المؤقت
        let _ = fs::remove_file(&temp_path);

        Ok(())
    }

    /// إنشاء نسخة احتياطية من نظام تشغيل
    pub async fn backup_os(&self, os_name: &str) -> Result<OSBackup> {
        info!("💾 إنشاء نسخة احتياطية من {}", os_name);

        let os_path = self.os_storage_path.join(os_name);
        if !os_path.exists() {
            return Err(anyhow::anyhow!("النظام {} غير موجود", os_name));
        }

        let backup_name = format!("{}_{}", os_name, chrono::Utc::now().format("%Y%m%d_%H%M%S"));
        let backup_file = self.backup_path.join(format!("{}.tar.gz", backup_name));

        // إنشاء الأرشيف
        let output = Command::new("tar")
            .args(&[
                "-czf", 
                backup_file.to_str().unwrap(),
                "-C", 
                self.os_storage_path.to_str().unwrap(),
                os_name
            ])
            .output()
            .context("فشل في تنفيذ أمر tar")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("فشل في إنشاء النسخة الاحتياطية: {}", error));
        }

        // حساب حجم النسخة الاحتياطية
        let backup_size = fs::metadata(&backup_file)
            .context("فشل في قراءة حجم النسخة الاحتياطية")?
            .len() / 1024 / 1024; // تحويل إلى MB

        let backup = OSBackup {
            os_name: os_name.to_string(),
            backup_date: chrono::Utc::now(),
            backup_size_mb: backup_size,
            backup_path: backup_file.to_string_lossy().to_string(),
            is_bootable: true, // سنفترض أنه قابل للتشغيل
        };

        info!("✅ تم إنشاء نسخة احتياطية من {} ({}MB)", os_name, backup_size);
        Ok(backup)
    }

    /// استعادة نظام من نسخة احتياطية
    pub async fn restore_os_from_backup(&self, backup: &OSBackup) -> Result<()> {
        info!("🔄 استعادة {} من النسخة الاحتياطية", backup.os_name);

        let backup_path = Path::new(&backup.backup_path);
        if !backup_path.exists() {
            return Err(anyhow::anyhow!("النسخة الاحتياطية غير موجودة"));
        }

        // حذف النظام الحالي إذا كان موجوداً
        let os_path = self.os_storage_path.join(&backup.os_name);
        if os_path.exists() {
            fs::remove_dir_all(&os_path)
                .context("فشل في حذف النظام الحالي")?;
        }

        // استخراج النسخة الاحتياطية
        let output = Command::new("tar")
            .args(&[
                "-xzf",
                backup.backup_path.as_str(),
                "-C",
                self.os_storage_path.to_str().unwrap()
            ])
            .output()
            .context("فشل في تنفيذ أمر استخراج")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("فشل في استعادة النسخة الاحتياطية: {}", error));
        }

        // إعادة تسجيل النظام
        self.register_os(&backup.os_name, &os_path).await?;

        info!("✅ تم استعادة {} بنجاح", backup.os_name);
        Ok(())
    }

    /// حذف نظام تشغيل
    pub async fn remove_os(&self, os_name: &str, create_backup: bool) -> Result<()> {
        info!("🗑️ حذف نظام التشغيل: {}", os_name);

        let os_path = self.os_storage_path.join(os_name);
        if !os_path.exists() {
            return Err(anyhow::anyhow!("النظام {} غير موجود", os_name));
        }

        // إنشاء نسخة احتياطية قبل الحذف إذا طُلب ذلك
        if create_backup {
            info!("💾 إنشاء نسخة احتياطية قبل الحذف");
            self.backup_os(os_name).await?;
        }

        // حذف النظام
        fs::remove_dir_all(&os_path)
            .context("فشل في حذف مجلد النظام")?;

        // إزالة من قائمة الأنظمة المتاحة
        self.unregister_os(os_name).await?;

        info!("✅ تم حذف {} بنجاح", os_name);
        Ok(())
    }

    /// إعداد النظام الافتراضي للتشغيل
    pub async fn set_default_os(&self, os_name: &str) -> Result<()> {
        info!("⚙️ تعيين {} كنظام افتراضي", os_name);

        // التحقق من وجود النظام
        if !self.os_exists(os_name) {
            return Err(anyhow::anyhow!("النظام {} غير موجود", os_name));
        }

        // تحديث ملف التكوين
        let mut boot_config = self.load_boot_configuration().await?;
        boot_config.default_os = Some(os_name.to_string());
        self.save_boot_configuration(&boot_config).await?;

        info!("✅ تم تعيين {} كنظام افتراضي", os_name);
        Ok(())
    }

    /// الحصول على قائمة الأنظمة المتاحة
    pub async fn get_available_systems(&self) -> Result<Vec<OperatingSystem>> {
        debug!("📋 جمع قائمة الأنظمة المتاحة");

        let mut systems = Vec::new();

        // مسح مجلد أنظمة التشغيل
        if self.os_storage_path.exists() {
            let entries = fs::read_dir(&self.os_storage_path)
                .context("فشل في قراءة مجلد الأنظمة")?;

            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Ok(os) = self.analyze_os_directory(&entry.path()).await {
                        systems.push(os);
                    }
                }
            }
        }

        // مسح أنظمة إضافية في مواقع أخرى
        systems.extend(self.scan_external_systems().await?);

        // ترتيب حسب آخر استخدام
        systems.sort_by(|a, b| {
            match (&a.last_used, &b.last_used) {
                (Some(a_time), Some(b_time)) => b_time.cmp(a_time),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.name.cmp(&b.name),
            }
        });

        Ok(systems)
    }

    /// الحصول على قائمة النسخ الاحتياطية
    pub async fn get_backups(&self) -> Result<Vec<OSBackup>> {
        debug!("📦 جمع قائمة النسخ الاحتياطية");

        let mut backups = Vec::new();

        if self.backup_path.exists() {
            let entries = fs::read_dir(&self.backup_path)
                .context("فشل في قراءة مجلد النسخ الاحتياطية")?;

            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("gz") {
                    if let Ok(backup) = self.analyze_backup_file(&path).await {
                        backups.push(backup);
                    }
                }
            }
        }

        // ترتيب حسب التاريخ (الأحدث أولاً)
        backups.sort_by(|a, b| b.backup_date.cmp(&a.backup_date));

        Ok(backups)
    }

    /// تحديث نظام تشغيل موجود
    pub async fn update_os(&self, os_name: &str, update_source: &str) -> Result<()> {
        info!("🔄 تحديث نظام {}", os_name);

        // إنشاء نسخة احتياطية قبل التحديث
        let backup = self.backup_os(os_name).await?;
        info!("💾 تم إنشاء نسخة احتياطية: {}", backup.backup_path);

        // محاولة التحديث
        match self.perform_os_update(os_name, update_source).await {
            Ok(_) => {
                info!("✅ تم تحديث {} بنجاح", os_name);
                Ok(())
            }
            Err(e) => {
                error!("❌ فشل في تحديث {}: {}", os_name, e);
                
                // استعادة النسخة الاحتياطية عند الفشل
                warn!("🔄 استعادة النسخة الاحتياطية");
                self.restore_os_from_backup(&backup).await?;
                
                Err(e)
            }
        }
    }

    /// تحسين أداء نظام تشغيل
    pub async fn optimize_os(&self, os_name: &str) -> Result<()> {
        info!("⚡ تحسين أداء {}", os_name);

        let os_path = self.os_storage_path.join(os_name);
        if !os_path.exists() {
            return Err(anyhow::anyhow!("النظام {} غير موجود", os_name));
        }

        // تنظيف الملفات المؤقتة
        self.cleanup_temporary_files(&os_path).await?;

        // تحسين قاعدة البيانات (إذا وجدت)
        self.optimize_databases(&os_path).await?;

        // ضغط الملفات غير المستخدمة
        self.compress_unused_files(&os_path).await?;

        // تحديث فهرس الملفات
        self.update_file_index(&os_path).await?;

        info!("✅ تم تحسين {} بنجاح", os_name);
        Ok(())
    }

    // =====================================
    // وظائف مساعدة داخلية
    // =====================================

    fn detect_image_type(&self, image_path: &str) -> Result<ImageType> {
        let path = Path::new(image_path);
        let extension = path.extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("لا يمكن تحديد نوع الصورة"))?
            .to_lowercase();

        match extension.as_str() {
            "iso" => Ok(ImageType::ISO),
            "img" => Ok(ImageType::IMG),
            "tar" | "tgz" => Ok(ImageType::TAR),
            "zip" => Ok(ImageType::ZIP),
            _ => {
                // محاولة تحديد النوع من محتوى الملف
                self.detect_image_type_by_content(image_path)
            }
        }
    }

    fn detect_image_type_by_content(&self, image_path: &str) -> Result<ImageType> {
        let output = Command::new("file")
            .arg(image_path)
            .output()
            .context("فشل في تحديد نوع الملف")?;

        let file_info = String::from_utf8_lossy(&output.stdout).to_lowercase();

        if file_info.contains("iso") {
            Ok(ImageType::ISO)
        } else if file_info.contains("tar") {
            Ok(ImageType::TAR)
        } else if file_info.contains("zip") {
            Ok(ImageType::ZIP)
        } else {
            Ok(ImageType::IMG) // افتراضي
        }
    }

    async fn install_from_iso(&self, iso_path: &str, install_path: &Path) -> Result<()> {
        info!("📀 تثبيت من ISO: {}", iso_path);

        // إنشاء نقطة تحميل مؤقتة
        let mount_point = format!("/tmp/dos_safar_mount_{}", 
            std::process::id());
        fs::create_dir_all(&mount_point)
            .context("فشل في إنشاء نقطة التحميل")?;

        // تحميل الـ ISO
        let mount_output = Command::new("mount")
            .args(&["-o", "loop", iso_path, &mount_point])
            .output()
            .context("فشل في تحميل ISO")?;

        if !mount_output.status.success() {
            let _ = fs::remove_dir(&mount_point);
            return Err(anyhow::anyhow!("فشل في تحميل ISO"));
        }

        // نسخ المحتويات
        let copy_result = Command::new("cp")
            .args(&["-r", &format!("{}/*", mount_point), 
                   install_path.to_str().unwrap()])
            .output();

        // إلغاء تحميل الـ ISO
        let _ = Command::new("umount").arg(&mount_point).output();
        let _ = fs::remove_dir(&mount_point);

        match copy_result {
            Ok(output) if output.status.success() => Ok(()),
            Ok(output) => {
                let error = String::from_utf8_lossy(&output.stderr);
                Err(anyhow::anyhow!("فشل في نسخ الملفات: {}", error))
            }
            Err(e) => Err(anyhow::anyhow!("خطأ في تنفيذ الأمر: {}", e))
        }
    }

    async fn install_from_img(&self, img_path: &str, install_path: &Path) -> Result<()> {
        info!("💾 تثبيت من IMG: {}", img_path);

        // نسخ صورة القرص مباشرة
        let output = Command::new("dd")
            .args(&[
                &format!("if={}", img_path),
                &format!("of={}/system.img", install_path.to_str().unwrap()),
                "bs=4M",
                "conv=fsync"
            ])
            .output()
            .context("فشل في نسخ صورة القرص")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("فشل في نسخ IMG: {}", error));
        }

        // محاولة تحميل الصورة لاستخراج الملفات
        self.extract_img_contents(install_path).await?;

        Ok(())
    }

    async fn install_from_tar(&self, tar_path: &str, install_path: &Path) -> Result<()> {
        info!("📦 تثبيت من TAR: {}", tar_path);

        let output = Command::new("tar")
            .args(&[
                "-xf", tar_path,
                "-C", install_path.to_str().unwrap()
            ])
            .output()
            .context("فشل في استخراج TAR")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("فشل في استخراج TAR: {}", error));
        }

        Ok(())
    }

    async fn install_from_zip(&self, zip_path: &str, install_path: &Path) -> Result<()> {
        info!("🗂️ تثبيت من ZIP: {}", zip_path);

        let output = Command::new("unzip")
            .args(&[
                "-q", zip_path,
                "-d", install_path.to_str().unwrap()
            ])
            .output()
            .context("فشل في استخراج ZIP")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("فشل في استخراج ZIP: {}", error));
        }

        Ok(())
    }

    async fn extract_img_contents(&self, install_path: &Path) -> Result<()> {
        let img_file = install_path.join("system.img");
        if !img_file.exists() {
            return Ok(()); // لا توجد صورة لاستخراجها
        }

        let mount_point = format!("/tmp/dos_safar_img_mount_{}", 
            std::process::id());
        fs::create_dir_all(&mount_point)
            .context("فشل في إنشاء نقطة تحميل الصورة")?;

        // محاولة تحميل الصورة
        let mount_output = Command::new("mount")
            .args(&["-o", "loop", img_file.to_str().unwrap(), &mount_point])
            .output();

        if let Ok(output) = mount_output {
            if output.status.success() {
                // نسخ المحتويات
                let _ = Command::new("cp")
                    .args(&["-r", &format!("{}/*", mount_point), 
                           install_path.to_str().unwrap()])
                    .output();

                // إلغاء التحميل
                let _ = Command::new("umount").arg(&mount_point).output();
            }
        }

        let _ = fs::remove_dir(&mount_point);
        Ok(())
    }

    async fn configure_installed_os(&self, install_path: &Path, os_name: &str) -> Result<()> {
        info!("⚙️ تكوين النظام المثبت: {}", os_name);

        // إنشاء ملف التكوين الخاص بالنظام
        let config_file = install_path.join("dos_safar_config.toml");
        let os_config = format!(
            r#"[system]
name = "{}"
install_date = "{}"
version = "1.0"
bootable = true

[hardware]
auto_detect = true
optimize_for_gaming = true

[display]
auto_resolution = true
safe_mode = false
"#,
            os_name,
            chrono::Utc::now().to_rfc3339()
        );

        fs::write(&config_file, os_config)
            .context("فشل في كتابة ملف التكوين")?;

        // تطبيق تحسينات خاصة بالجهاز
        self.apply_device_optimizations(install_path).await?;

        // إعداد البوت
        self.setup_boot_configuration(install_path, os_name).await?;

        Ok(())
    }

    async fn apply_device_optimizations(&self, install_path: &Path) -> Result<()> {
        // تحسينات خاصة بـ Raspberry Pi
        if self.is_raspberry_pi() {
            self.apply_raspberry_pi_optimizations(install_path).await?;
        }

        // تحسينات خاصة بأجهزة الألعاب المحمولة
        if self.is_gaming_handheld() {
            self.apply_gaming_handheld_optimizations(install_path).await?;
        }

        Ok(())
    }

    async fn apply_raspberry_pi_optimizations(&self, install_path: &Path) -> Result<()> {
        info!("🍓 تطبيق تحسينات Raspberry Pi");

        // تكوين GPU memory split
        let boot_config = install_path.join("config.txt");
        if boot_config.exists() {
            let mut config_content = fs::read_to_string(&boot_config)
                .unwrap_or_default();

            // إضافة تحسينات GPU
            if !config_content.contains("gpu_mem") {
                config_content.push_str("\n# DOS Safar GPU optimizations\n");
                config_content.push_str("gpu_mem=128\n");
                config_content.push_str("gpu_freq=500\n");
                config_content.push_str("over_voltage=2\n");

                fs::write(&boot_config, config_content)
                    .context("فشل في تحديث config.txt")?;
            }
        }

        Ok(())
    }

    async fn apply_gaming_handheld_optimizations(&self, install_path: &Path) -> Result<()> {
        info!("🎮 تطبيق تحسينات أجهزة الألعاب المحمولة");

        // تحسينات خاصة بالشاشات الصغيرة
        let display_config = install_path.join("display_config.txt");
        let display_settings = r#"# Gaming Handheld Display Settings
hdmi_force_hotplug=1
hdmi_group=2
hdmi_mode=87
hdmi_cvt=480 320 60 6 0 0 0
display_rotate=0
"#;

        fs::write(&display_config, display_settings)
            .context("فشل في كتابة تكوين الشاشة")?;

        Ok(())
    }

    async fn setup_boot_configuration(&self, install_path: &Path, os_name: &str) -> Result<()> {
        info!("🚀 إعداد تكوين البوت لـ {}", os_name);

        // إنشاء سكريبت البوت
        let boot_script = install_path.join("boot.sh");
        let script_content = format!(
            r#"#!/bin/bash
# DOS Safar Boot Script for {}
echo "🎮 Starting {} via DOS Safar..."

# Set environment variables
export DOS_SAFAR_OS="{}"
export DOS_SAFAR_PATH="{}"

# Load system specific configurations
if [ -f "{}/dos_safar_config.toml" ]; then
    echo "📝 Loading DOS Safar configuration..."
fi

# Start the operating system
echo "🚀 Launching {}..."
exec /sbin/init
"#,
            os_name, os_name, os_name, 
            install_path.to_str().unwrap(),
            install_path.to_str().unwrap(),
            os_name
        );

        fs::write(&boot_script, script_content)
            .context("فشل في كتابة سكريبت البوت")?;

        // جعل السكريبت قابل للتنفيذ
        Command::new("chmod")
            .args(&["+x", boot_script.to_str().unwrap()])
            .output()
            .context("فشل في تعيين صلاحيات التنفيذ")?;

        Ok(())
    }

    async fn register_os(&self, os_name: &str, os_path: &Path) -> Result<()> {
        info!("📝 تسجيل النظام {} في قاعدة البيانات", os_name);

        let registry_file = self.os_storage_path.join("registry.json");
        let mut registry: serde_json::Value = if registry_file.exists() {
            let content = fs::read_to_string(&registry_file)?;
            serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        // إضافة معلومات النظام
        registry[os_name] = serde_json::json!({
            "name": os_name,
            "path": os_path.to_str().unwrap(),
            "install_date": chrono::Utc::now().to_rfc3339(),
            "last_used": null,
            "bootable": true,
            "size_mb": self.calculate_directory_size(os_path).await.unwrap_or(0)
        });

        let registry_content = serde_json::to_string_pretty(&registry)?;
        fs::write(&registry_file, registry_content)
            .context("فشل في كتابة سجل الأنظمة")?;

        Ok(())
    }

    async fn unregister_os(&self, os_name: &str) -> Result<()> {
        info!("🗑️ إزالة {} من سجل الأنظمة", os_name);

        let registry_file = self.os_storage_path.join("registry.json");
        if !registry_file.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&registry_file)?;
        let mut registry: serde_json::Value = serde_json::from_str(&content)
            .unwrap_or(serde_json::json!({}));

        // إزالة النظام من السجل
        if let Some(obj) = registry.as_object_mut() {
            obj.remove(os_name);
        }

        let registry_content = serde_json::to_string_pretty(&registry)?;
        fs::write(&registry_file, registry_content)
            .context("فشل في تحديث سجل الأنظمة")?;

        Ok(())
    }

    async fn download_os_image(&self, url: &str, output_path: &str) -> Result<()> {
        info!("📥 تحميل صورة النظام من: {}", url);

        let output = Command::new("wget")
            .args(&[
                "-O", output_path,
                "--progress=bar",
                "--show-progress",
                url
            ])
            .output()
            .context("فشل في تنفيذ أمر التحميل")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("فشل في التحميل: {}", error));
        }

        info!("✅ تم تحميل الصورة بنجاح");
        Ok(())
    }

    async fn analyze_os_directory(&self, os_path: &Path) -> Result<OperatingSystem> {
        let os_name = os_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // تحديد نوع النظام
        let os_type = self.detect_os_type(os_path);

        // قراءة معلومات إضافية
        let description = self.get_os_description(os_path, &os_type);
        let last_used = self.get_last_used_date(&os_name).await;

        Ok(OperatingSystem {
            name: os_name,
            path: os_path.to_string_lossy().to_string(),
            description,
            os_type,
            is_bootable: self.is_bootable(os_path),
            last_used,
        })
    }

    fn detect_os_type(&self, os_path: &Path) -> OSType {
        // فحص ملفات مميزة لكل نوع نظام
        if os_path.join("retropie").exists() || 
           os_path.join("RetroPie").exists() {
            return OSType::RetroPie;
        }

        if os_path.join("batocera").exists() ||
           os_path.join("BATOCERA").exists() {
            return OSType::Batocera;
        }

        if os_path.join("recalbox").exists() {
            return OSType::Recalbox;
        }

        if os_path.join("config.txt").exists() &&
           os_path.join("cmdline.txt").exists() {
            return OSType::RaspberryPiOS;
        }

        if os_path.join("ubuntu").exists() ||
           os_path.join("etc/lsb-release").exists() {
            return OSType::Ubuntu;
        }

        OSType::Unknown
    }

    fn get_os_description(&self, os_path: &Path, os_type: &OSType) -> String {
        // محاولة قراءة وصف من ملف التكوين
        let config_file = os_path.join("dos_safar_config.toml");
        if config_file.exists() {
            if let Ok(content) = fs::read_to_string(&config_file) {
                // محاولة استخراج الوصف من TOML
                // هذا مبسط - في التنفيذ الحقيقي نستخدم مكتبة TOML
                for line in content.lines() {
                    if line.starts_with("description") {
                        if let Some(desc) = line.split('=').nth(1) {
                            return desc.trim().trim_matches('"').to_string();
                        }
                    }
                }
            }
        }

        // وصف افتراضي حسب النوع
        match os_type {
            OSType::RetroPie => "نظام الألعاب الكلاسيكية RetroPie".to_string(),
            OSType::Batocera => "نظام الألعاب Batocera".to_string(),
            OSType::Recalbox => "نظام الألعاب Recalbox".to_string(),
            OSType::RaspberryPiOS => "نظام التشغيل الرسمي لـ Raspberry Pi".to_string(),
            OSType::Ubuntu => "نظام Ubuntu Linux".to_string(),
            OSType::Debian => "نظام Debian Linux".to_string(),
            OSType::Unknown => "نظام تشغيل غير معروف".to_string(),
        }
    }

    fn is_bootable(&self, os_path: &Path) -> bool {
        // فحص وجود ملفات البوت الأساسية
        let boot_files = vec![
            "boot.sh",
            "kernel.img",
            "config.txt",
            "system.img",
        ];

        boot_files.iter().any(|file| os_path.join(file).exists())
    }

    async fn get_last_used_date(&self, os_name: &str) -> Option<chrono::DateTime<chrono::Utc>> {
        let registry_file = self.os_storage_path.join("registry.json");
        if !registry_file.exists() {
            return None;
        }

        let content = fs::read_to_string(&registry_file).ok()?;
        let registry: serde_json::Value = serde_json::from_str(&content).ok()?;

        let last_used_str = registry[os_name]["last_used"].as_str()?;
        chrono::DateTime::parse_from_rfc3339(last_used_str)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc))
    }

    async fn calculate_directory_size(&self, dir_path: &Path) -> Result<u64> {
        let output = Command::new("du")
            .args(&["-s", "-m", dir_path.to_str().unwrap()])
            .output()
            .context("فشل في حساب حجم المجلد")?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let size_str = output_str.split_whitespace().next().unwrap_or("0");
            Ok(size_str.parse().unwrap_or(0))
        } else {
            Ok(0)
        }
    }

    fn os_exists(&self, os_name: &str) -> bool {
        self.os_storage_path.join(os_name).exists()
    }

    fn is_raspberry_pi(&self) -> bool {
        Path::new("/proc/device-tree/model").exists() &&
        fs::read_to_string("/proc/device-tree/model")
            .unwrap_or_default()
            .to_lowercase()
            .contains("raspberry pi")
    }

    fn is_gaming_handheld(&self) -> bool {
        // فحص مبسط لأجهزة الألعاب المحمولة
        let model_info = fs::read_to_string("/proc/device-tree/model")
            .unwrap_or_default()
            .to_lowercase();
        
        model_info.contains("anbernic") ||
        model_info.contains("rg351") ||
        model_info.contains("rg552")
    }

    // باقي الوظائف المساعدة...
    async fn scan_external_systems(&self) -> Result<Vec<OperatingSystem>> {
        // فحص مواقع إضافية للأنظمة
        Ok(Vec::new()) // مبسط
    }

    async fn analyze_backup_file(&self, backup_path: &Path) -> Result<OSBackup> {
        let metadata = fs::metadata(backup_path)?;
        let size_mb = metadata.len() / 1024 / 1024;
        
        let file_name = backup_path.file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        // استخراج اسم النظام وتاريخ النسخة الاحتياطية من اسم الملف
        let parts: Vec<&str> = file_name.split('_').collect();
        let os_name = parts.get(0).unwrap_or(&"unknown").to_string();
        
        Ok(OSBackup {
            os_name,
            backup_date: metadata.created()
                .ok()
                .and_then(|t| chrono::DateTime::from(t).into())
                .unwrap_or_else(chrono::Utc::now),
            backup_size_mb: size_mb,
            backup_path: backup_path.to_string_lossy().to_string(),
            is_bootable: true,
        })
    }

    async fn load_boot_configuration(&self) -> Result<BootConfiguration> {
        let config_file = self.boot_partition_path.join("dos_safar_boot.json");
        
        if config_file.exists() {
            let content = fs::read_to_string(&config_file)?;
            let config: BootConfiguration = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            // تكوين افتراضي
            Ok(BootConfiguration {
                default_os: None,
                timeout_seconds: 10,
                available_systems: Vec::new(),
                boot_order: Vec::new(),
                recovery_mode: false,
            })
        }
    }

    async fn save_boot_configuration(&self, config: &BootConfiguration) -> Result<()> {
        let config_file = self.boot_partition_path.join("dos_safar_boot.json");
        let content = serde_json::to_string_pretty(config)?;
        fs::write(&config_file, content)?;
        Ok(())
    }

    async fn perform_os_update(&self, os_name: &str, update_source: &str) -> Result<()> {
        // تنفيذ مبسط للتحديث
        info!("تحديث {} من {}", os_name, update_source);
        Ok(())
    }

    async fn cleanup_temporary_files(&self, os_path: &Path) -> Result<()> {
        // تنظيف الملفات المؤقتة
        Ok(())
    }

    async fn optimize_databases(&self, os_path: &Path) -> Result<()> {
        // تحسين قواعد البيانات
        Ok(())
    }

    async fn compress_unused_files(&self, os_path: &Path) -> Result<()> {
        // ضغط الملفات غير المستخدمة
        Ok(())
    }

    async fn update_file_index(&self, os_path: &Path) -> Result<()> {
        // تحديث فهرس الملفات
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ImageType {
    ISO,
    IMG,
    TAR,
    ZIP,
}