section main {

  option graphics: Enum {
      @default: High,
      @values: [Low, Medium, High]
  }

  option audio: Int {
      @default: 42,
  }

  section localization {
      @default: $lang_en

    option language: String {
        @default: "en_US"
    }
  }
}
