// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Italian translation table.

use crate::i18n::TextKey;

pub const fn it_t(key: TextKey) -> &'static str {
    match key {
        TextKey::Share => "Condividi",
        TextKey::Copy => "Copia",
        TextKey::Copied => "Copiato!",
        TextKey::CopyToClipboard => "Copia negli appunti",
        TextKey::Notice => "Nota",
        TextKey::Error => "Errore",
        TextKey::DismissError => "Chiudi errore",
        TextKey::FiltersShow => "Mostra filtri",
        TextKey::FiltersHide => "Nascondi filtri",
        TextKey::Language => "Lingua",
        TextKey::PageTitle => "Esploratore LOTUS di dati aperti collegati",
        TextKey::DarkModeToggle => "Attiva/disattiva tema chiaro/scuro",
        TextKey::DarkMode => "Scuro",
        TextKey::LightMode => "Chiaro",
        TextKey::GoToHomepage => "Vai alla home page",
        TextKey::SkipToResults => "Vai al contenuto principale",
        TextKey::PageSubtitle => {
            "Esplora dati aperti collegati: prodotti naturali, organismi e letteratura scientifica."
        }
        TextKey::ResolvedTaxon => "Taxon risolto",
        TextKey::QueryHash => "Hash della query",
        TextKey::ResultHash => "Hash del risultato",
        TextKey::CopyTaxonQid => "Copia QID del taxon",
        TextKey::CopyFullQueryHash => "Copia hash completo della query (SHA-256)",
        TextKey::CopyFullResultHash => "Copia hash completo del risultato (SHA-256)",
        TextKey::CopyShareableLink => "Copia link condivisibile",
        TextKey::Unique => "Uniche",
        TextKey::LoadingTitle => "Interrogazione di Wikidata tramite QLever...",
        TextKey::LoadingHint => {
            "I set di risultati di grandi dimensioni possono richiedere alcuni secondi."
        }
        TextKey::LoadingResolvingTaxon => "Risoluzione del taxon...",
        TextKey::LoadingFetchingResults => "Recupero risultati...",
        TextKey::LoadingProcessingResults => "Elaborazione dei conteggi risultati...",
        TextKey::LoadingRendering => "Rendering della tabella...",
        TextKey::Retry => "Riprova",
        TextKey::ErrorHintValidation => "Controlla l'input e riprova.",
        TextKey::ErrorHintConfiguration => {
            "In questo ambiente manca la configurazione di servizio richiesta."
        }
        TextKey::ErrorHintNetwork => "Problema di rete rilevato. Riprova.",
        TextKey::ErrorHintRateLimit => {
            "Limite di richieste raggiunto sul servizio upstream. Attendi circa un minuto e riprova."
        }
        TextKey::ErrorHintBadRequest => {
            "Il server ha rifiutato la richiesta. Controlla i parametri di ricerca."
        }
        TextKey::ErrorHintParse => {
            "Impossibile interpretare la risposta. Riprova o affina la query."
        }
        TextKey::ErrorHintUnknown => "Errore inatteso. Riprova.",
        TextKey::WelcomeLeadA => {
            "Questa applicazione dimostra la potenza dei dati aperti collegati collegando i prodotti naturali agli organismi e alla letteratura scientifica. "
        }
        TextKey::WelcomeLeadB => {
            "Il modello di dati collega gli composti, i taxa e i riferimenti—provenienti da "
        }
        TextKey::WelcomeLeadC => ", pubblicati come dati aperti collegati su ",
        TextKey::WelcomeLeadD => " e interrogati tramite SPARQL tramite ",
        TextKey::WelcomeLeadE => ".",
        TextKey::ExampleGentiana => {
            "Inserisci un nome di taxon, un QID Wikidata o * per tutti i taxa"
        }
        TextKey::ExampleSmilesOnly => "Incolla uno SMILES o un Molfile nel campo struttura",
        TextKey::ExampleQueryExecute => "Esegui",
        TextKey::ExampleQueryTaxon => "Scarica CSV",
        TextKey::ExampleQueryStructure => "Scarica JSON",
        TextKey::ExampleQueryAdvanced => "Scarica RDF",
        TextKey::ExampleApiUrls => "Esempi di URL API",
        TextKey::LabelLanguagePolicy => {
            "Le etichette usano prima 'mul' e poi 'en' per risultati confrontabili."
        }
        TextKey::SearchFilters => "Filtri di ricerca",
        TextKey::Taxon => "Taxon",
        TextKey::TaxonPlaceholder => "Gentiana lutea - Q34317 - *",
        TextKey::TaxonHint => "Nome, QID oppure * per tutti i taxa.",
        TextKey::StructureSmilesOrMol => "SMILES o Molfile",
        TextKey::StructurePlaceholder => {
            "c1ccccc1   - oppure incolla un blocco Molfile (V2000 / V3000)"
        }
        TextKey::Substructure => "Sottostruttura",
        TextKey::Similarity => "Somiglianza",
        TextKey::StructureSearchMode => "Modalità di ricerca struttura",
        TextKey::EditCopyDaylightSmiles => "Modifica -> Copia come SMILES Daylight",
        TextKey::CopyExtendedSmilesMol => "Copia come SMILES estesi / MOL V3000",
        TextKey::FormulaFilter => "Filtro formula",
        TextKey::ExactFormula => "Formula bruta",
        TextKey::MinCount => "min",
        TextKey::MaxCount => "max",
        TextKey::MinCountAria => "conteggio minimo",
        TextKey::MaxCountAria => "conteggio massimo",
        TextKey::ElementRequirement => "vincolo",
        TextKey::ElementStateAllowed => "consentito",
        TextKey::ElementStateRequired => "richiesto",
        TextKey::ElementStateExcluded => "escluso",
        TextKey::Search => "Cerca",
        TextKey::Searching => "Ricerca...",
        TextKey::MolecularMass => "Massa molecolare (Da)",
        TextKey::Min => "Min",
        TextKey::Max => "Max",
        TextKey::PublicationYear => "Anno di pubblicazione",
        TextKey::YearFrom => "Da",
        TextKey::YearTo => "A",
        TextKey::RunSearch => "Avvia ricerca",
        TextKey::KetcherSummary => "Editor di strutture (Ketcher)",
        TextKey::KetcherHintA => "Devi disegnare o cercare una struttura? Apri la scheda ",
        TextKey::KetcherHintB => " e poi copia con ",
        TextKey::KetcherHintC => " (oppure ",
        TextKey::KetcherHintD => ") e usalo nel campo struttura della scheda Ricerca.",
        TextKey::KetcherIframeTitle => "Editor di strutture Ketcher",
        TextKey::KetcherClickToLoad => "Clicca per caricare l'editor di strutture Ketcher.",
        TextKey::KindNoteSmiles => "  Inviato come letterale SPARQL su una singola riga.",
        TextKey::KindNoteMol2000 => {
            "  Inoltrato senza modifiche a SACHEM scoredSubstructureSearch."
        }
        TextKey::KindNoteMol3000 => {
            "  Inoltrato senza modifiche a SACHEM scoredSubstructureSearch (CTAB v3000)."
        }
        TextKey::DatasetStatistics => "Statistiche del dataset",
        TextKey::DownloadResults => "Scarica i risultati",
        TextKey::PreparingDownload => "Preparazione download...",
        TextKey::StartingCsvDownload => "Avvio download CSV...",
        TextKey::PreparingJsonDownload => "Preparazione download JSON...",
        TextKey::PreparingRdfDownload => "Preparazione download RDF...",
        TextKey::DownloadCsvTitle => "Scarica i risultati in CSV",
        TextKey::DownloadCsvLabel => "Scarica CSV",
        TextKey::DownloadJsonTitle => "Scarica i risultati in JSON",
        TextKey::DownloadJsonLabel => "Scarica JSON",
        TextKey::DownloadRdfTitle => "Scarica i risultati in RDF (Turtle)",
        TextKey::DownloadRdfLabel => "Scarica RDF",
        TextKey::DownloadMetadataTitle => "Scarica metadati Schema.org (JSON-LD)",
        TextKey::DownloadMetadataLabel => "Scarica metadati",
        TextKey::OpenInQlever => "Apri in QLever",
        TextKey::OpenInQleverTitle => "Apri questa query nell'interfaccia web di QLever",
        TextKey::OpenInEndpoint => "Apri nell'endpoint",
        TextKey::OpenInEndpointTitle => {
            "Apri questa query nell'interfaccia web dell'endpoint SPARQL"
        }
        TextKey::NoResults => "Nessun risultato. Prova ad ampliare la ricerca.",
        TextKey::StageTaxonSearch => "risoluzione del taxon",
        TextKey::StageResultsQuery => "recupero risultati",
        TextKey::DisplayCappedHint => {
            "Per sicurezza di memoria su questo dispositivo vengono mostrate solo le prime righe. I conteggi restano esatti."
        }
        TextKey::Structure => "Struttura",
        TextKey::Compound => "Composto",
        TextKey::Mass => "Massa",
        TextKey::Formula => "Formula",
        TextKey::TaxonCol => "Taxon",
        TextKey::Reference => "Riferimento",
        TextKey::Year => "Anno",
        TextKey::FooterData => "Dati",
        TextKey::FooterCitation => "Citazione",
        TextKey::FooterCode => "Codice",
        TextKey::FooterArchive => "Archivio",
        TextKey::FooterPrograms => "Programmi",
        TextKey::FooterLicense => "Licenza",
        TextKey::FooterForData => " per i dati ",
        TextKey::FooterForCode => " per il codice",
        TextKey::TableTriplesAria => "Triple composto-taxon-riferimento",
        TextKey::OpenFullSizeDepiction => "Apri la rappresentazione a dimensione piena",
        TextKey::OpenInWikidata => "Apri in Wikidata",
        TextKey::OpenInScholia => "Apri in Scholia",
        TextKey::OpenInCompoundScholia => "Apri il composto in Scholia",
        TextKey::OpenInTaxonScholia => "Apri il taxon in Scholia",
        TextKey::OpenInReferenceScholia => "Apri il riferimento in Scholia",
        TextKey::OpenDoi => "Apri DOI",
        TextKey::Statement => "Dichiarazione",
        TextKey::SparqlQuery => "Query SPARQL",
        TextKey::CopySparqlQuery => "Copia query SPARQL",
    }
}
