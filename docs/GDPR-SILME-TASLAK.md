# GDPR / Veri Silme Tasarımı — TASLAK (onay + avukat görüşü bekliyor)

> Durum: taslak 16 Eyl 2026. Dış denetim (§4.8–4.10) haklı: public ≠ kişisel-dışı,
> embedding ≠ gizli, permanent index × silme hakkı çelişir. İçte mekanizma YOK.
> Bu belge asgari tasarımı önerir.

## Veri haritası (silme yayılmak zorunda)
1. Kaynak metin: corpus DB'leri (`madde` tablosu) + shard'lar (arşivde).
2. Vektörler: merged DB `vektor` (118G) + emb arşivleri (109G, salt-okunur yedek).
3. Index: FAISS + `.ids.npy` (madde_id → vektör eşlemesi).
4. Türevler: FTS5 indexi, sorgu logları, site istatistikleri.

## Önerilen mekanizma (3 katman)
1. **Tombstone (anlık):** `silinen(madde_id, neden, ts)` tablosu tüm canlı DB'lerde;
   sorgu yolu (`sorgu.py`, `/api/ara`) tombstone'lu ID'leri filtreler. Etki: arama
   sonuçlarından düşer (dakikalar).
2. **Periyodik rebuild (aylık):** FAISS ilgili shard için yeniden kurulur
   (tombstone'lar hariç); ids.npy yenilenir. Etki: vektör index'ten çıkar.
3. **Arşiv notu:** salt-okunur arşivler (emb/shard) periyotta yeniden yazılmaz;
   her arşiv zarfına `SILINENLER-README` eklenir, restore'ta tombstone uygulanır.
   Tam fiziksel silme yılda bir arşiv devirinde.

## Corpus lisans ilkesi (mevcut + sıkılaştırma)
- Mevcut ayrım korunur: Wikipedia CC BY-SA, Tatoeba CC BY, Gutenberg PD,
  WikiHow non-commercial (ücretli katman dışı).
- Ek: her corpus kaydına `lisans + TDM-rezervasyon-durumu + ticari-kullanim`
  alanı; rezervasyonlu kaynaklar Faz 2 havuzuna girmez.

## PII asgari önlemleri (embed öncesi)
- E-posta/telefon/TC-kimlik kalıpları için regex filtresi parse aşamasında
  (`temizle()` yanına `pii_maskele()`); eşleşen madde `atlanan` tablosuna
  `neden='pii'` ile işlenir (mekanizma mevcut, kural eklenecek).
- Hassas kategori (sağlık/siyasi/çocuk) için corpus-düzeyi bayrak + inceleme kuyruğu.

## Açık kararlar (operatör + avukat)
- Veri sorumlusu kimliği + başvuru kanalı (siteye eklenecek).
- Saklama süreleri (loglar: 90 gün önerisi).
- 0.9 avukat brief'ine bu taslak eklensin mi? (öneri: evet)
