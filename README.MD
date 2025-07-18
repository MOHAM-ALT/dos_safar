# HackberryPi5 DietPi Complete Setup Tool

## نظرة عامة على المشروع

هذا المشروع عبارة عن **أداة إعداد شاملة ومؤتمتة** لجهاز HackberryPi5، وهو جهاز حاسوب محمول مفتوح المصدر يعتمد على Raspberry Pi 5. الأداة مصممة لتبسيط عملية تثبيت وتكوين نظام التشغيل DietPi مع جميع الإعدادات المطلوبة لضمان عمل جميع مكونات الأجهزة بشكل مثالي.

### الهدف من المشروع

إنشاء **برنامج Windows تلقائي** يقوم بـ:
- تحميل صورة DietPi المناسبة لـ Raspberry Pi 5
- حرق النظام على بطاقة SD
- تطبيق **جميع التكوينات المطلوبة** لـ HackberryPi5
- ضمان عمل الشاشة اللمسية ولوحة المفاتيح والصوت والمكونات الأخرى

## مواصفات جهاز HackberryPi5

### الأجهزة المدعومة في هذا المشروع

#### المعالج والذاكرة
- **المعالج**: Raspberry Pi 5 (Broadcom BCM2712)
- **الذاكرة**: 4GB/8GB/16GB LPDDR4X
- **التخزين**: بطاقة SD (8GB كحد أدنى، 32GB مستحسن)

#### الشاشة واللمس
- **الشاشة**: 4 بوصة TFT IPS
- **الدقة**: 720×720 بكسل (مربعة)
- **نوع اللمس**: شاشة لمسية مقاومة (Resistive Touch)
- **وحدة التحكم باللمس**: XPT2046 أو ADS7846
- **الواجهة**: SPI للتحكم، RGB متوازي للبيانات
- **معدل التحديث**: 60Hz

#### لوحة المفاتيح
- **النوع**: لوحة مفاتيح BlackBerry أصلية (9900 أو Q20)
- **الواجهة**: I2C أو USB
- **الدعم**: VIAL للتخصيص
- **الميزات**: مفاتيح وظيفية، أزرار اختصار

#### الصوت والاتصال
- **الصوت**: مكبرات صوت مدمجة عبر Bluetooth
- **البلوتوث**: مدمج في Pi 5
- **WiFi**: 802.11ac dual-band مدمج
- **منافذ USB**: USB-A و USB-C

#### الطاقة والبطارية
- **البطارية**: 2× 18650 ليثيوم أيون
- **السعة**: 6000mAh إجمالية
- **مدة التشغيل**: 3-4 ساعات
- **الشحن**: USB-C، 2A
- **إدارة الطاقة**: I2C battery management

#### المدخلات والمخرجات
- **I2C**: مفعل للمستشعرات ومنفذ Stemma
- **SPI**: مفعل للشاشة والتحكم
- **GPIO**: وصول كامل للتوسعات
- **UART**: مفعل للتطوير والتصحيح

## التحديات التقنية المطلوب حلها

### 1. تكوين الشاشة اللمسية
**المشكلة**: الشاشة 720×720 تتطلب برامج تشغيل مخصصة
**الحل المطلوب**:
```bash
# إعدادات config.txt للشاشة
dtparam=spi=on
dtoverlay=fbtft,spi0-0,ili9486,width=720,height=720,rotate=0,fps=60,bgr=1
dtparam=cs=0,dc=22,reset=27,led=18
```

### 2. تكوين اللمس
**المشكلة**: تحتاج معايرة دقيقة للاستجابة الصحيحة
**الحل المطلوب**:
```bash
# إعدادات اللمس
dtoverlay=ads7846,cs=1,penirq=25,penirq_pull=2,speed=50000,keep_vref_on=0,swapxy=0,pmax=255,xohms=150,xmin=200,xmax=3900,ymin=200,ymax=3900
```

### 3. تكوين الصوت والبلوتوث
**المشكلة**: الصوت يجب أن يعمل عبر البلوتوث فقط
**الحل المطلوب**:
- تكوين PulseAudio للبلوتوث
- إقران تلقائي للمكبرات
- تعطيل الصوت السلكي

### 4. إعدادات الطاقة
**المشكلة**: إدارة البطارية عبر I2C
**الحل المطلوب**:
- مراقبة جهد البطارية
- تحذيرات البطارية المنخفضة
- إدارة الشحن

### 5. تكوين لوحة المفاتيح
**المشكلة**: لوحة BlackBerry تحتاج إعدادات خاصة
**الحل المطلوب**:
- تثبيت VIAL
- تكوين I2C للتحكم
- خرائط المفاتيح المخصصة

## متطلبات النظام للأداة

### متطلبات Windows
- **نظام التشغيل**: Windows 10/11 (64-bit)
- **الصلاحيات**: تشغيل كمدير (Administrator)
- **المساحة الحرة**: 5GB كحد أدنى
- **اتصال الإنترنت**: مطلوب لتحميل DietPi

### البرامج المطلوبة (يتم تحميلها تلقائياً)
- **7-Zip**: لاستخراج ملفات النظام
- **dd للويندوز**: لحرق الصور
- **PowerShell**: للتحميل والتكوين

### الأجهزة المطلوبة
- **بطاقة SD**: 8GB كحد أدنى، 32GB مستحسن
- **قارئ بطاقات SD**: USB أو مدمج
- **HackberryPi5**: الجهاز المستهدف

## الميزات الرئيسية للأداة

### 1. التحميل التلقائي
- تحميل أحدث إصدار من DietPi لـ Pi 5
- التحقق من سلامة الملفات
- إعادة المحاولة في حالة فشل التحميل
- دعم Resume للتحميلات المتقطعة

### 2. حرق النظام
- اكتشاف تلقائي لبطاقة SD
- تحذيرات الأمان لمنع حذف البيانات الخطأ
- شريط التقدم لمتابعة عملية الحرق
- التحقق من نجاح العملية

### 3. التكوين الشامل
**إعدادات config.txt**:
- تكوين الشاشة والدقة
- إعدادات اللمس والمعايرة
- تفعيل I2C وSPI
- إعدادات الصوت والبلوتوث
- تحسينات الأداء والطاقة

**إعدادات الشبكة**:
- تكوين WiFi مع الشبكة المحددة
- تفعيل SSH للوصول عن بعد
- إعدادات أمان أساسية

**إعدادات النظام**:
- كلمة مرور افتراضية محددة
- المنطقة الزمنية (السعودية)
- اللغة والتخطيط (عربي/إنجليزي)
- تثبيت الأدوات الأساسية

### 4. التحقق والاختبار
- فحص شامل لجميع الملفات المُنشأة
- التحقق من صحة الإعدادات
- تقرير مفصل عن حالة التكوين
- اقتراحات لحل المشاكل

## سير العمل التفصيلي للأداة

### المرحلة 1: التحضير والفحص
```
1. فحص صلاحيات المدير
2. إنشاء مجلد العمل المؤقت
3. فحص توفر الأدوات المطلوبة
4. تحميل الأدوات المفقودة (7-Zip, dd)
5. اكتشاف بطاقات SD المتاحة
```

### المرحلة 2: تحميل النظام
```
1. تحديد رابط تحميل DietPi الأحدث
2. التحقق من المساحة المتاحة
3. تحميل الملف مع شريط التقدم
4. فحص سلامة الملف (الحجم والنوع)
5. استخراج صورة النظام
```

### المرحلة 3: حرق النظام
```
1. عرض بطاقات SD المتاحة
2. طلب تأكيد من المستخدم
3. فحص حالة البطاقة (لا تحتوي على نظام مهم)
4. حرق الصورة مع مراقبة التقدم
5. التحقق من نجاح عملية الحرق
```

### المرحلة 4: التكوين الشامل
```
1. اكتشاف بطاقة SD المحروقة
2. إنشاء ملفات التكوين:
   - config.txt (إعدادات الأجهزة)
   - dietpi.txt (إعدادات النظام)
   - dietpi-wifi.txt (إعدادات الشبكة)
   - wpa_supplicant.conf (WiFi)
   - modules (برامج التشغيل)
   - ssh (تفعيل SSH)
3. إنشاء سكريبت ما بعد التثبيت
4. إنشاء ملفات التوثيق والمرجع
```

### المرحلة 5: التحقق والتقرير
```
1. فحص وجود جميع الملفات المطلوبة
2. التحقق من صحة المحتوى
3. اختبار قابلية القراءة
4. إنشاء تقرير التحقق النهائي
5. تنظيف الملفات المؤقتة
```

## الملفات والإعدادات المُنشأة

### 1. ملف config.txt الشامل
```ini
# HackberryPi5 Complete Configuration
# ====================================

# Core System Settings
arm_boost=1
over_voltage=2
arm_freq=2400
gpu_freq=750
gpu_mem=128

# Display Configuration - 720x720 TFT
hdmi_blanking=2
hdmi_ignore_hotplug=1
dtparam=spi=on
dtoverlay=spi0-hw-cs

# TFT Display Driver (ILI9486/ILI9488 compatible)
dtoverlay=fbtft,spi0-0,ili9486,width=720,height=720,rotate=0,fps=60,bgr=1
dtparam=cs=0,dc=22,reset=27,led=18

# Touch Screen Support (XPT2046/ADS7846)
dtoverlay=ads7846,cs=1,penirq=25,penirq_pull=2,speed=50000,keep_vref_on=0,swapxy=0,pmax=255,xohms=150,xmin=200,xmax=3900,ymin=200,ymax=3900

# I2C Configuration for Sensors and Stemma Port
dtparam=i2c_arm=on
dtparam=i2c1=on
dtparam=i2c_arm_baudrate=400000

# Audio Configuration for Built-in Speakers
dtparam=audio=on
audio_pwm_mode=2
disable_audio_dither=1

# Bluetooth Configuration
dtparam=krnbt=on

# UART for Debugging and Keyboard
enable_uart=1

# Power Management for Battery
dtparam=pwr_led_trigger=mmc0
dtparam=act_led_trigger=heartbeat

# Keyboard Configuration
dtoverlay=hid-generic

# Disable unnecessary features to save power
camera_auto_detect=0
display_auto_detect=0
```

### 2. ملف dietpi.txt للنظام
```ini
# HackberryPi5 DietPi Configuration
AUTO_SETUP_AUTOMATED=1
AUTO_SETUP_GLOBAL_PASSWORD=dietpi123

# Regional settings
AUTO_SETUP_TIMEZONE=Asia/Riyadh
AUTO_SETUP_LOCALE=en_US.UTF-8
AUTO_SETUP_KEYBOARD_LAYOUT=us

# Network settings
AUTO_SETUP_NET_ETHERNET_ENABLED=1
AUTO_SETUP_NET_WIFI_ENABLED=1

# SSH settings
AUTO_SETUP_SSH_SERVER_INDEX=1

# Display settings for HackberryPi5
AUTO_SETUP_HEADLESS=0
AUTO_SETUP_AUTOSTART_TARGET_INDEX=7

# Development tools installation
AUTO_SETUP_INSTALL_SOFTWARE_ID=17  # Git
AUTO_SETUP_INSTALL_SOFTWARE_ID=130 # Python
AUTO_SETUP_INSTALL_SOFTWARE_ID=9   # Build tools
AUTO_SETUP_INSTALL_SOFTWARE_ID=16  # Desktop environment
AUTO_SETUP_INSTALL_SOFTWARE_ID=103 # Node.js

AUTO_SETUP_BOOT_WAIT_FOR_NETWORK=2
SURVEY_OPTED_IN=0
```

### 3. ملف dietpi-wifi.txt للشبكة
```ini
# HackberryPi5 WiFi Configuration
aWIFI_COUNTRYCODE[0]='SA'
aWIFI_SSID[0]='A7rf'
aWIFI_KEY[0]='11111111aA'
aWIFI_KEYMGR[0]='WPA-PSK'
aWIFI_ENABLED=1
```

### 4. ملف modules لبرامج التشغيل
```
# HackberryPi5 Kernel Modules
spi-bcm2835
fbtft_device
fbtft
fb_ili9486
ads7846
evdev
uinput
```

### 5. سكريبت ما بعد التثبيت
```bash
#!/bin/bash
# HackberryPi5 Post-Install Configuration Script

echo "Starting HackberryPi5 post-install setup..."

# Update system
apt update && apt upgrade -y

# Install essential packages
apt install -y git vim nano htop curl wget unzip
apt install -y build-essential python3-pip python3-dev
apt install -y nodejs npm
apt install -y i2c-tools spi-tools

# Install touch screen utilities
apt install -y xinput xinput-calibrator
apt install -y xserver-xorg-input-evdev

# Install display utilities
apt install -y xrandr fbset

# Configure touchscreen
modprobe ads7846
modprobe evdev

# Set up GPIO access
usermod -a -G gpio,spi,i2c,audio,video dietpi

# Configure display settings
export DISPLAY=:0
export FRAMEBUFFER=/dev/fb1

# Create touchscreen calibration
mkdir -p /etc/X11/xorg.conf.d/
cat > /etc/X11/xorg.conf.d/99-calibration.conf << 'EOF'
Section "InputClass"
    Identifier "calibration"
    MatchProduct "ADS7846 Touchscreen"
    Option "Calibration" "200 3900 200 3900"
    Option "SwapAxes" "0"
    Option "InvertX" "0"
    Option "InvertY" "0"
EndSection
EOF

# Set up framebuffer for console
echo 'FRAMEBUFFER=/dev/fb1' >> /etc/environment

# Configure keyboard
dpkg-reconfigure -f noninteractive keyboard-configuration

# Install additional drivers
apt install -y device-tree-compiler

echo "HackberryPi5 setup completed successfully!"
```

## الاختبارات والتحقق

### اختبارات الأجهزة
```bash
# فحص الشاشة
fbset -fb /dev/fb1

# فحص اللمس
evtest /dev/input/event0

# فحص I2C
i2cdetect -y 1

# فحص SPI
ls /dev/spi*

# فحص WiFi
iwconfig

# فحص البلوتوث
bluetoothctl show
```

### اختبارات النظام
- فحص إقلاع النظام بدون أخطاء
- اختبار استجابة الشاشة اللمسية
- اختبار اتصال WiFi والبلوتوث
- فحص عمل لوحة المفاتيح
- اختبار إدارة البطارية

## استكشاف الأخطاء وحلها

### مشاكل الشاشة
**المشكلة**: الشاشة لا تعمل أو تظهر فارغة
**الحلول**:
1. فحص اتصالات SPI
2. التحقق من برامج التشغيل في config.txt
3. استخدام إعداد بديل: `dtoverlay=waveshare35a`
4. فحص سجلات النظام: `/var/log/Xorg.0.log`

### مشاكل اللمس
**المشكلة**: اللمس لا يستجيب أو غير دقيق
**الحلول**:
1. تشغيل أداة المعايرة: `sudo xinput_calibrator`
2. فحص إعدادات ads7846
3. اختبار جهاز اللمس: `evtest`
4. تحديث ملف المعايرة

### مشاكل الشبكة
**المشكلة**: عدم اتصال WiFi
**الحلول**:
1. فحص قوة الإشارة
2. التحقق من إعدادات wpa_supplicant
3. إعادة تشغيل خدمة الشبكة
4. فحص logs: `journalctl -u wpa_supplicant`

### مشاكل الصوت
**المشكلة**: عدم عمل الصوت عبر البلوتوث
**الحلول**:
1. إعادة إقران الجهاز
2. فحص إعدادات PulseAudio
3. تحديث برامج التشغيل
4. استخدام أدوات التشخيص

## التخصيص والتطوير

### تخصيص الإعدادات
المستخدمون يمكنهم تعديل:
- شبكة WiFi وكلمة المرور
- المنطقة الزمنية واللغة
- إعدادات الشاشة والدوران
- خرائط لوحة المفاتيح
- البرامج المثبتة افتراضياً

### إضافة ميزات جديدة
- دعم شاشات أخرى
- إعدادات بطارية مخصصة
- أنماط طاقة متقدمة
- تكامل مع خدمات سحابية

## الأمان والخصوصية

### الإعدادات الأمنية الافتراضية
- تغيير كلمة المرور الافتراضية
- تفعيل جدار الحماية
- تحديثات أمنية تلقائية
- تشفير اتصالات SSH

### توصيات الأمان
- استخدام مفاتيح SSH بدلاً من كلمات المرور
- تفعيل المصادقة ثنائية العامل
- مراقبة سجلات النظام
- تحديث النظام بانتظام

## الدعم والمجتمع

### الموارد الرسمية
- **موقع المطور**: ZitaoTech
- **GitHub**: github.com/ZitaoTech/HackberryPi5
- **Tindie Store**: متجر القطع والأجهزة
- **Discord Server**: مجتمع المطورين

### المراجع التقنية
- دليل Raspberry Pi 5 الرسمي
- وثائق DietPi
- مراجع SPI وI2C
- دلائل تطوير Linux المدمج

## الخلاصة

هذه الأداة تهدف إلى جعل عملية إعداد HackberryPi5 سهلة ومؤتمتة بالكامل، مع ضمان عمل جميع المكونات بشكل مثالي من أول تشغيل. الأداة تتضمن جميع التكوينات المطلوبة بناءً على المواصفات الرسمية للجهاز والمتطلبات التقنية المحددة.

**الهدف النهائي**: إنشاء تجربة "Plug and Play" حقيقية لمستخدمي HackberryPi5، حيث يمكن للمستخدم حرق النظام وتشغيل الجهاز مباشرة بدون الحاجة لأي تكوينات إضافية.