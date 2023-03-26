package detection.files.msi

import com.sun.jna.Native
import com.sun.jna.Platform

class ProductLanguage(languageCode: Int) {
    val locale = languageCode.toLanguageTag()

    private fun Int.toLanguageTag() = if (Platform.isWindows()) getLanguageTagWindows() else lcidMap[this]

    private fun Int.getLanguageTagWindows(): String? {
        val lcidLibrary = LCIDLibrary.INSTANCE

        val lpLanguage = CharArray(localeBufferSize)
        var result = lcidLibrary.GetLocaleInfoW(this, LOCALE_SABBREVLANGNAME, lpLanguage, lpLanguage.size)
        if (result > 0) {
            val language = Native.toString(lpLanguage).dropLast(1)
            val lpCountry = CharArray(localeBufferSize)
            result = lcidLibrary.GetLocaleInfoW(this, LOCALE_SABBREVCTRYNAME, lpCountry, lpCountry.size)
            if (result > 0) {
                val country = Native.toString(lpCountry).dropLast(1)
                return language.lowercase() + "-" + country.uppercase()
            }
        }
        return null
    }

    companion object {
        private const val localeBufferSize = 4 // 3 Characters (e.g. USA) + null terminator
        private const val LOCALE_SABBREVLANGNAME = 0x3
        private const val LOCALE_SABBREVCTRYNAME = 0x7
        private val lcidMap = hashMapOf(
            1078 to "af", 1052 to "sq", 1118 to "am", 5121 to "ar-DZ", 15361 to "ar-BH", 3073 to "ar-EG",
            2049 to "ar-IQ", 11265 to "ar-JO", 13313 to "ar-KW", 12289 to "ar-LB", 4097 to "ar-LY", 6145 to "ar-MA",
            8193 to "ar-OM", 16385 to "ar-QA", 1025 to "ar-SA", 10241 to "ar-SY", 7169 to "ar-TN", 14337 to "ar-AE",
            9217 to "ar-YE", 1067 to "hy", 1101 to "as", 2092 to "az-AZ", 1068 to "az-AZ", 1069 to "eu", 1059 to "be",
            2117 to "bn", 1093 to "bn", 5146 to "bs", 1026 to "bg", 1109 to "my", 1027 to "ca", 2052 to "zh-CN",
            3076 to "zh-HK", 5124 to "zh-MO", 4100 to "zh-SG", 1028 to "zh-TW", 1050 to "hr", 1029 to "cs",
            1030 to "da", 1125 to "dv", 2067 to "nl-BE", 1043 to "nl-NL", 3081 to "en-AU", 10249 to "en-BZ",
            4105 to "en-CA", 9225 to "en-CB", 2057 to "en-GB", 16393 to "en-IN", 6153 to "en-IE", 8201 to "en-JM",
            5129 to "en-NZ", 13321 to "en-PH", 7177 to "en-ZA", 11273 to "en-TT", 1033 to "en-US", 12297 to "en",
            1061 to "et", 1080 to "fo", 1065 to "fa", 464 to "1124", 1035 to "fi", 2060 to "fr-BE", 11276 to "fr",
            3084 to "fr-CA", 9228 to "fr", 12300 to "fr", 1036 to "fr-FR", 5132 to "fr-LU", 13324 to "fr", 6156 to "fr",
            14348 to "fr", 10252 to "fr", 4108 to "fr-CH", 7180 to "fr", 462 to "1122", 1071 to "mk", 2108 to "gd-IE",
            1084 to "gd", 1110 to "gl", 1079 to "ka", 3079 to "de-AT", 1031 to "de-DE", 5127 to "de-LI",
            4103 to "de-LU", 2055 to "de-CH", 1032 to "el", 1140 to "gn", 1095 to "gu", 1037 to "he", 1081 to "hi",
            1038 to "hu", 1039 to "is", 470 to "1136", 1057 to "id", 1040 to "it-IT", 2064 to "it-CH", 1041 to "ja",
            1099 to "kn", 1120 to "ks", 1087 to "kk", 1107 to "km", 457 to "1111", 1042 to "ko", 1108 to "lo",
            1142 to "la", 1062 to "lv", 1063 to "lt", 2110 to "ms-BN", 1086 to "ms-MY", 1100 to "ml", 1082 to "mt",
            1153 to "mi", 1102 to "mr", 2128 to "mn", 1104 to "mn", 1121 to "ne", 1044 to "no-NO", 2068 to "no-NO",
            1096 to "or", 1045 to "pl", 1046 to "pt-BR", 2070 to "pt-PT", 1094 to "pa", 1047 to "rm", 2072 to "ro-MO",
            1048 to "ro", 1049 to "ru", 2073 to "ru-MO", 1103 to "sa", 3098 to "sr-SP", 2074 to "sr-SP", 1074 to "tn",
            1113 to "sd", 1115 to "si", 1051 to "sk", 1060 to "sl", 1143 to "so", 1070 to "sb", 11274 to "es-AR",
            16394 to "es-BO", 13322 to "es-CL", 9226 to "es-CO", 5130 to "es-CR", 7178 to "es-DO", 12298 to "es-EC",
            17418 to "es-SV", 4106 to "es-GT", 18442 to "es-HN", 2058 to "es-MX", 19466 to "es-NI", 6154 to "es-PA",
            15370 to "es-PY", 10250 to "es-PE", 20490 to "es-PR", 1034 to "es-ES", 14346 to "es-UY", 8202 to "es-VE",
            1089 to "sw", 2077 to "sv-FI", 1053 to "sv-SE", 1064 to "tg", 1097 to "ta", 1092 to "tt", 1098 to "te",
            1054 to "th", 1105 to "bo", 1073 to "ts", 1055 to "tr", 1090 to "tk", 1058 to "uk", 1056 to "ur",
            2115 to "uz-UZ", 1091 to "uz-UZ", 1066 to "vi", 1106 to "cy", 1076 to "xh", 1085 to "yi", 1077 to "zu"
        )
    }
}
