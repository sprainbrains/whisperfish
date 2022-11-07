use libsignal_service::proto::AttachmentPointer;
use whisperfish::store::{temp, NewMessage};
use whisperfish::store::{Storage, StorageLocation};

pub type InMemoryDb = (Storage, StorageLocation<tempdir::TempDir>);

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

pub fn storage() -> InMemoryDb {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    rt.block_on(async {
        let temp = temp();
        (
            Storage::new(&temp, None, 12345, "Some Password", [0; 52], None)
                .await
                .expect("Failed to initalize storage"),
            temp,
        )
    })
}

fn fetch_augmented_messages(c: &mut Criterion) {
    let mut group = c.benchmark_group("fetch_augmented_messages");
    group.significance_level(0.05).sample_size(20);
    for elements in (9..18).map(|x| 1 << x) {
        group.throughput(Throughput::Elements(elements));
        for attachments in 0..3 {
            // for receipts in (0..6) {
            let (mut storage, _loc) = storage();
            // Insert `elements` messages
            let session = storage.fetch_or_insert_session_by_e164("+32474");
            for _ in 0..elements {
                let (msg, _) = storage.process_message(
                    NewMessage {
                        session_id: Some(session.id),
                        source_e164: Some("+32474".into()),
                        source_uuid: None,
                        text: "Foo bar".into(),
                        timestamp: chrono::Utc::now().naive_utc(),
                        sent: false,
                        received: false,
                        is_read: false,
                        flags: 0,
                        attachment: None,
                        mime_type: None,
                        has_attachment: false,
                        outgoing: false,
                    },
                    None,
                );
                for _attachment in 0..attachments {
                    storage.register_attachment(msg.id, AttachmentPointer::default(), "");
                }
                // for _receipt in 0..receipts {
                //     storage.register_attachment(msg.id, "", "");
                // }
            }
            group.bench_with_input(
                BenchmarkId::from_parameter(format!(
                    "{} messages/{} attachments",
                    elements, attachments
                )),
                &elements,
                move |b, _| {
                    // Now benchmark the retrieve function
                    b.iter(|| black_box(storage.fetch_all_messages_augmented(session.id)))
                },
            );
            // }
        }
    }
    group.finish();
}

criterion_group!(benches, fetch_augmented_messages);
criterion_main!(benches);
