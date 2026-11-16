# Tinta Texto — Preview 4

> **Status:** 2026-11-16 · draft for the Preview 4 release (10 December 2026), version 0.18.1, with
> the 16 November upstream sync · **Revision history:** 2026-11-16 created (changes from Preview 3,
> 12 November, to #73). Updated at each weekly sync until the freeze on 7 December.

Release notes for the GitHub release and the site, in Spanish, Ukrainian and English. Changes since
Preview 3 (v0.15.1, 12 November 2026). Before publishing: build the Windows installers with
`packaging/windows/tinta-build-release.ps1` (in pdfcraft) and run the smoke test on them.

## Español

**Tinta Texto — vista previa 4.** Versión de prueba: úsela con copias de sus documentos.

Novedades:

- **Ajustar a una página** (Disposición ▸ Configurar página): reduce un poco la letra y el espaciado
  para que un documento que se pasa de una página quepa en una.
- **Documentos de Word más fieles:** las tablas que siguen en otra página, sus bordes y sus filas de
  altura mínima, los números de las listas, el superíndice de los documentos antiguos y las versalitas
  quedan como en Word; un .docx que no fija el control de líneas viudas y huérfanas se pagina como
  en Word.
- **Propiedades del documento:** el idioma, el estado («Final»…), el identificador y la versión se
  conservan al guardar, y el archivo vuelve a decir que lo guardó Tinta Texto.
- **Archivos Markdown (.md):** una vista para editar el texto fuente.
- **Pegar** respeta lo que otro programa dejó en el portapapeles; velocidad de desplazamiento ajustable
  para el ratón y el panel táctil.
- **Idiomas:** interfaz en bosnio; separación silábica en checo y rumano con el paquete de
  diccionarios adicionales.
- **Linux:** funciona en Wayland aunque falten las bibliotecas de X11.
- Del proyecto original (WordCraft): muchas correcciones de diseño, portapapeles y desplazamiento.

Limitaciones conocidas: algunos documentos complejos de Word (gráficos y SmartArt editables, ciertas
tablas flotantes) aún no se ven exactamente igual; la revisión ortográfica de catalán, gallego y
euskera necesita sus diccionarios (incluidos).

## Українська

**Tinta Texto — попередня версія 4.** Тестова версія: працюйте з копіями документів.

Що нового:

- **Вмістити на одну сторінку** (Макет ▸ Параметри сторінки): трохи зменшує шрифт і інтервали, щоб
  документ, який ледь виходить за сторінку, вмістився на одній.
- **Документи Word ближчі до оригіналу:** таблиці, що переходять на наступну сторінку, їхні межі
  й рядки з мінімальною висотою, номери списків, верхній індекс у старих документах і малі великі
  літери виглядають як у Word; .docx, у якому не задано контроль висячих рядків, розбивається на
  сторінки як у Word.
- **Властивості документа:** мова, стан («Остаточний»…), ідентифікатор і версія зберігаються, а у
  файлі знову записано, що його зберіг Tinta Texto.
- **Файли Markdown (.md):** режим редагування вихідного тексту.
- **Вставлення** враховує, що інша програма поклала в буфер обміну; швидкість прокручування
  налаштовується окремо для миші й тачпада.
- **Мови:** інтерфейс боснійською; перенесення слів чеською й румунською з додатковим пакетом
  словників.
- **Linux:** працює у Wayland, навіть якщо немає бібліотек X11.
- З основного проєкту (WordCraft): багато виправлень макета, буфера обміну й прокручування.

Відомі обмеження: деякі складні документи Word (редаговані діаграми й SmartArt, окремі плаваючі
таблиці) ще виглядають не зовсім так само; перевірка правопису каталанською, галісійською й
баскською потребує їхніх словників (вони в комплекті).

## English

**Tinta Texto — Preview 4.** A test version: work on copies of your documents.

What's new:

- **Fit One Page** (Layout ▸ Page Setup): shrinks the text and spacing a little so a document that
  just runs over one page fits on it.
- **Word documents closer to Word:** tables continued on the next page, their borders and rows with
  a minimum height, list numbers, superscript in older documents and small caps look as they do in
  Word; a .docx that doesn't set widow and orphan control breaks into pages as in Word.
- **Document properties:** the language, status ("Final"…), identifier and version are kept when
  you save, and the file says again that Tinta Texto saved it.
- **Markdown files (.md):** a view to edit the source text.
- **Paste** respects what another program left on the clipboard; scrolling speed can be set for the
  mouse and the touchpad.
- **Languages:** a Bosnian interface; Czech and Romanian hyphenation with the extra dictionary pack.
- **Linux:** runs on Wayland even without the X11 libraries.
- From the original project (WordCraft): many layout, clipboard and scrolling fixes.

Known limits: some complex Word documents (editable charts and SmartArt, some floating tables) don't
look exactly the same yet; Catalan, Galician and Basque spelling needs their dictionaries (included).
