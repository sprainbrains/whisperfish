use crate::platform::QQmlEngine;
use cpp::cpp;
use qttypes::QString;

pub fn install(app: &mut QQmlEngine) {
    cpp!(unsafe [app as "QQmlEngine *"] {
        app->addImageProvider(QLatin1String("blurhash"), new BlurhashImageProvider);
    });
}

cpp! {{
    #include <QtQuick/QQuickImageProvider>

    class BlurhashImageProvider : public QQuickImageProvider
    {
    public:
        BlurhashImageProvider()
                   : QQuickImageProvider(QQuickImageProvider::Image)
        {
        }

        QImage requestImage(const QString &id, QSize *size, const QSize &requestedSize) override
        {
            int width = 100;
            int height = 100;

            if (size)
               *size = QSize(width, height);

            QImage img(requestedSize.width() > 0 ? requestedSize.width() : width,
                           requestedSize.height() > 0 ? requestedSize.height() : height,
                           QImage::Format::Format_RGBX8888);
            uchar *buf = img.bits();
            // size_t size = img.sizeInBytes(); // Qt 5.10+
            size_t size_in_bytes = img.byteCount();
            width = img.width();
            height = img.height();

            rust!(WF_decode_blurhash [
                id : &QString as "const QString &",
                buf : *mut u8 as "uchar *",
                width : u32 as "int",
                height : u32 as "int",
                size_in_bytes : usize as "size_t"
            ] -> i32 as "int" {
                let id = id.to_string();
                // XXX We might want some *real* error handling at some point.
                if id == "null" {
                    return -1;
                }
                let id = percent_encoding::percent_decode_str(&id).decode_utf8().unwrap();
                let img = blurhash::decode(id.as_ref(), width, height, 1.0);
                assert_eq!(img.len(), size_in_bytes);
                let slice = unsafe { std::slice::from_raw_parts_mut(buf, size_in_bytes) };
                slice.copy_from_slice(&img);
                0
            });

            return img;
        }
    };
} }
