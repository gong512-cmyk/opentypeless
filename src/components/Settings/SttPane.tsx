import { useTranslation } from 'react-i18next'
import { useAppStore } from '../../stores/appStore'
import { useAuthStore } from '../../stores/authStore'
import { STT_PROVIDERS, LANGUAGES } from '../../lib/constants'
import { benchSttConnection } from '../../lib/tauri'
import { FormField } from './shared/FormField'
import { CheckCircle2, XCircle, Loader2, Crown } from 'lucide-react'

export function SttPane() {
  const config = useAppStore((s) => s.config)
  const updateConfig = useAppStore((s) => s.updateConfig)
  const sttTestStatus = useAppStore((s) => s.sttTestStatus)
  const setSttTestStatus = useAppStore((s) => s.setSttTestStatus)
  const sttLatencyMs = useAppStore((s) => s.sttLatencyMs)
  const setSttLatencyMs = useAppStore((s) => s.setSttLatencyMs)
  const { user, plan } = useAuthStore()
  const { t } = useTranslation()

  const isCloud = config.stt_provider === 'cloud'
  const isCustomWhisper = config.stt_provider === 'custom-whisper'
  const isXiaomiMimo = config.stt_provider === 'xiaomi-mimo'
  const isConfigurableProvider = isCustomWhisper || isXiaomiMimo
  const xiaomiPreset = {
    endpoint: 'https://token-plan-cn.xiaomimimo.com/v1',
    model: 'mimo-v2.5',
  }

  const applyXiaomiPreset = () => {
    updateConfig({
      stt_provider: 'xiaomi-mimo',
      stt_base_url: xiaomiPreset.endpoint,
      stt_model: xiaomiPreset.model,
    })
    setSttTestStatus('idle')
    setSttLatencyMs(null)
  }

  const handleTest = async () => {
    setSttTestStatus('testing')
    setSttLatencyMs(null)
    try {
      const ms = await benchSttConnection(
        config.stt_api_key,
        config.stt_provider,
        config.stt_base_url,
        config.stt_model,
      )
      console.log('[STT Test] Received latency:', ms, 'type:', typeof ms)
      setSttLatencyMs(ms)
      setSttTestStatus('success')
    } catch (err) {
      console.error('[STT Test] Error:', err)
      setSttTestStatus('error')
    }
  }

  return (
    <div className="space-y-5">
      <FormField label={t('settings.provider')}>
        <select
          value={config.stt_provider}
          onChange={(e) => {
            updateConfig({ stt_provider: e.target.value as typeof config.stt_provider })
            setSttTestStatus('idle')
            setSttLatencyMs(null)
          }}
          className="w-full px-3 py-2.5 bg-bg-secondary border border-border rounded-[10px] text-[13px] text-text-primary outline-none focus:border-border-focus transition-colors"
        >
          {STT_PROVIDERS.map((p) => (
            <option key={p.value} value={p.value}>
              {p.label}
            </option>
          ))}
        </select>
      </FormField>

      {isCloud ? (
        <div className="border border-border rounded-[10px] px-3 py-3 space-y-2">
          <div className="flex items-center gap-2 text-[13px]">
            <Crown size={14} className="text-accent" />
            <span className="text-text-primary font-medium">{t('settings.cloudSttPro')}</span>
          </div>
          {!user ? (
            <p className="text-[12px] text-text-secondary">{t('settings.sttSignInHint')}</p>
          ) : plan !== 'pro' ? (
            <p className="text-[12px] text-text-secondary">{t('settings.sttUpgradeHint')}</p>
          ) : (
            <p className="text-[12px] text-green-500">{t('settings.sttProActive')}</p>
          )}
        </div>
      ) : (
        <>
          <FormField label={t('settings.apiKey')}>
            <div className="flex gap-2">
              <input
                type="password"
                value={config.stt_api_key}
                onChange={(e) => {
                  updateConfig({ stt_api_key: e.target.value })
                  setSttTestStatus('idle')
                  setSttLatencyMs(null)
                }}
                placeholder={t('settings.enterApiKey')}
                className="flex-1 px-3 py-2.5 bg-bg-secondary border border-border rounded-[10px] text-[13px] text-text-primary outline-none focus:border-border-focus transition-colors"
              />
              <button
                onClick={handleTest}
                disabled={
                  !config.stt_api_key ||
                  sttTestStatus === 'testing' ||
                  (isCustomWhisper && (!config.stt_base_url.trim() || !config.stt_model.trim()))
                }
                className="px-4 py-2.5 bg-accent text-white rounded-[10px] text-[13px] border-none cursor-pointer hover:bg-accent-hover disabled:opacity-40 disabled:cursor-not-allowed transition-colors flex items-center gap-1.5"
              >
                {sttTestStatus === 'testing' && <Loader2 size={14} className="animate-spin" />}
                {t('settings.test')}
              </button>
            </div>
            {sttTestStatus === 'success' && (
              <p className="flex items-center gap-1 text-[12px] text-success mt-2">
                <CheckCircle2 size={13} />{' '}
                {sttLatencyMs !== null ? `${sttLatencyMs}ms` : t('settings.connectionSuccess')}
              </p>
            )}
            {sttTestStatus === 'error' && (
              <p className="flex items-center gap-1 text-[12px] text-error mt-2">
                <XCircle size={13} /> {t('settings.connectionFailed')}
              </p>
            )}
            <p className="text-[11px] text-text-tertiary mt-1.5">{t('settings.storedLocally')}</p>
          </FormField>

          {isConfigurableProvider && (
            <>
              <div className="flex justify-end -mt-2">
                <button
                  type="button"
                  onClick={applyXiaomiPreset}
                  className="px-3 py-1.5 bg-bg-secondary border border-border rounded-[8px] text-[12px] text-text-secondary cursor-pointer hover:text-text-primary hover:border-border-focus transition-colors"
                >
                  {t('settings.xiaomiPreset')}
                </button>
              </div>
              <p className="text-[11px] text-text-tertiary -mt-1">
                {isXiaomiMimo ? t('settings.xiaomiPresetHintCompatible') : t('settings.xiaomiPresetHint')}
              </p>
              <FormField label={t('settings.sttEndpoint')}>
                <input
                  type="text"
                  value={config.stt_base_url}
                  onChange={(e) => {
                    updateConfig({ stt_base_url: e.target.value })
                    setSttTestStatus('idle')
                    setSttLatencyMs(null)
                  }}
                  placeholder="https://your-provider.com/v1/audio/transcriptions"
                  className="w-full px-3 py-2.5 bg-bg-secondary border border-border rounded-[10px] text-[13px] text-text-primary outline-none focus:border-border-focus transition-colors"
                />
              </FormField>
              <FormField label={t('settings.model')}>
                <input
                  type="text"
                  value={config.stt_model}
                  onChange={(e) => {
                    updateConfig({ stt_model: e.target.value })
                    setSttTestStatus('idle')
                    setSttLatencyMs(null)
                  }}
                  placeholder="whisper-1"
                  className="w-full px-3 py-2.5 bg-bg-secondary border border-border rounded-[10px] text-[13px] text-text-primary outline-none focus:border-border-focus transition-colors"
                />
              </FormField>
            </>
          )}
        </>
      )}

      <FormField label={t('settings.sttLanguage')}>
        <select
          value={config.stt_language}
          onChange={(e) => updateConfig({ stt_language: e.target.value })}
          className="w-full px-3 py-2.5 bg-bg-secondary border border-border rounded-[10px] text-[13px] text-text-primary outline-none focus:border-border-focus transition-colors"
        >
          {LANGUAGES.map((l) => (
            <option key={l.value} value={l.value}>
              {l.label}
            </option>
          ))}
        </select>
      </FormField>

      {config.stt_provider === 'local-whisper' && (
        <FormField label="LoRA Adapter (方言模型)">
          <input
            type="text"
            value={config.whisper_lora_path ?? ''}
            onChange={(e) => updateConfig({ whisper_lora_path: e.target.value })}
            placeholder="可选：方言 LoRA adapter 路径，如 models/cantonese.bin"
            className="w-full px-3 py-2.5 bg-bg-secondary border border-border rounded-[10px] text-[13px] text-text-primary outline-none focus:border-border-focus transition-colors"
          />
          <p className="text-[11px] text-text-tertiary mt-1">留空则使用基础模型。放入微调后的 .bin 文件路径即可启用方言识别。</p>
        </FormField>
      )}
    </div>
  )
}
