# ebs-snapshot-bill

Schat de GB-maandkosten van **Amazon EBS-snapshots** uit een lokale
`describe-snapshots`-JSON en toont oude of weesachtige (orphan) snapshots.
De tool werkt offline: hij leest alleen lokale bestanden en gebruikt geen netwerk.

Engelse versie: `README.md`. Tests en metingen: `QUALITY.nl.md`.

## Installatie

Versie 0.1.0. Bouwen vanaf de bron (Rust 1.85+), vanuit een checkout:

```sh
cargo install --path .
```

Zodra 0.1.0 op crates.io staat: `cargo install ebs-snapshot-bill`.

## Aan de slag in 5 minuten

1. Installeer de tool (zie Installatie).
2. Draai `ebs-snapshot-bill examples/normal.json`.
3. Voeg `--max-age-days 30` toe voor oudere snapshots, of `--json` voor JSON.
4. Exporteer je eigen gegevens met `aws ec2 describe-snapshots --owner-ids self > snaps.json`
   en draai de tool daarop.

## Opties

- `PATH...`: describe-snapshots-JSON-bestanden of een map met `*.json`.
- `--prices FILE`: prijsbestand. Standaard: ingebouwde us-east-1-tarieven (`prices/us-east-1.json`).
- `--volumes FILE`: describe-volumes-JSON; snapshots zonder volume daarin zijn orphans. Toegestaan: `{"Volumes":[{"VolumeId":"..."}]}`, een bovenste lijst `[{"VolumeId":"..."}]` (uitvoer van `describe-volumes --query 'Volumes[]'`) of een lijst id-teksten. Een lege lijst, een element zonder tekst-`VolumeId` of een dubbele `VolumeId`- of `Volumes`-sleutel is een fout (exit 2). Hetzelfde geldt voor een herhaalde sleutel ergens in een snapshotbestand of het prijsbestand en voor een herhaalde `SnapshotId` (ook als hetzelfde bestand twee keer wordt opgegeven).
- `--as-of DATE`: peildatum voor leeftijd, `YYYY-MM-DD`. Standaard 2026-11-09.
- `--max-age-days DAYS`: oudere snapshots staan als oud in de lijst. Standaard 90.
- `--json`: JSON-uitvoer (`schema_version` 1).

## Voorbeeld

```sh
ebs-snapshot-bill examples/old.json --max-age-days 30 --volumes tests/fixtures/volumes.json
```

Het rapport toont de geschatte GB-maandkosten, oude en orphan-snapshots en een
by_volume-overzicht.

## Afsluitcodes

- `0`: het rapport is gemaakt.
- `2`: fout in invoer of gebruik (bestand onleesbaar of ongeldig, foute optie, een `--volumes`-bestand in een onbekende vorm, een GB-totaal dat niet in 64 bits past, een herhaalde sleutel of een herhaalde `SnapshotId`). Er is geen verzonnen maximum: alleen een som die echt niet past geeft exit 2.

## Onderhoud

Prijzen veranderen. De tarieven in `prices/us-east-1.json` zijn een gedateerde
kopie (2026-11-09); controleer ze voor je op een getal vertrouwt. Patches komen
binnen zoals in `CONTRIBUTING.md` staat. Beveiligingsmeldingen: zie `SECURITY.md`.

## Prijzen en verantwoordelijkheid

De tarieven staan in `prices/us-east-1.json` (bron:
https://aws.amazon.com/ebs/pricing/, datum 2026-11-09). Dit is een schatting uit
een lokaal bestand, geen AWS-factuur. Jij controleert de tarieven en de uitkomst
tegen je eigen AWS-factuur voordat je erop handelt.

## Licentie

MIT OR Apache-2.0.

De licenties van de externe crates in het programma staan in [THIRD-PARTY-LICENSES.html](THIRD-PARTY-LICENSES.html).
