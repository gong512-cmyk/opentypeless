import { AnimatePresence, motion } from 'framer-motion'
import { useAppStore } from '../../stores/appStore'
import { saveOnboardingCompleted, updateConfig as saveConfig } from '../../lib/tauri'
import { OnboardingLayout } from './OnboardingLayout'
import { WelcomeStep } from './WelcomeStep'
import { SttSetupStep } from './SttSetupStep'
import { LlmSetupStep } from './LlmSetupStep'
import { DoneStep } from './DoneStep'
import { slideRight } from '../../lib/animations'

const TOTAL_STEPS = 5

export function Onboarding() {
  const step = useAppStore((s) => s.onboardingStep)
  const setStep = useAppStore((s) => s.setOnboardingStep)
  const setOnboardingCompleted = useAppStore((s) => s.setOnboardingCompleted)
  const sttTestStatus = useAppStore((s) => s.sttTestStatus)
  const llmTestStatus = useAppStore((s) => s.llmTestStatus)

  const canNext = (() => {
    switch (step) {
      case 0:
        return true // Welcome
      case 1:
        return sttTestStatus === 'success' // STT must pass
      case 2:
        return llmTestStatus === 'success' // LLM must pass
      case 3:
        return true // Done
      default:
        return false
    }
  })()

  const titles = [
    {
      title: '欢迎使用 OpenTypeless',
      subtitle: '几步设置即可开始语音输入',
    },
    {
      title: '语音识别 (STT)',
      subtitle: '配置语音转文字服务 — 推荐 SiliconFlow 中文识别最佳',
    },
    {
      title: 'AI 润色 (LLM)',
      subtitle: '配置大模型服务 — 推荐 DeepSeek 性价比最高',
    },
    { title: '设置完成', subtitle: undefined },
  ]

  const config = useAppStore((s) => s.config)

  const handleNext = async () => {
    if (step < TOTAL_STEPS - 1) {
      try {
        await saveConfig(config)
      } catch {
        // Best-effort save
      }
      setStep(step + 1)
    } else {
      await saveConfig(config)
      await saveOnboardingCompleted()
      setOnboardingCompleted(true)
    }
  }

  const handleBack = async () => {
    if (step > 0) {
      try {
        await saveConfig(config)
      } catch {
        // Best-effort save
      }
      setStep(step - 1)
    }
  }

  const handleSkip = async () => {
    await saveConfig(config)
    await saveOnboardingCompleted()
    setOnboardingCompleted(true)
  }

  return (
    <OnboardingLayout
      step={step}
      totalSteps={TOTAL_STEPS}
      title={titles[step].title}
      subtitle={titles[step].subtitle}
      canNext={canNext}
      canBack={step > 0}
      nextLabel={step === TOTAL_STEPS - 1 ? '开始使用' : '下一步'}
      onNext={handleNext}
      onBack={handleBack}
      onSkip={handleSkip}
    >
      <AnimatePresence mode="wait">
        <motion.div
          key={step}
          variants={slideRight}
          initial="initial"
          animate="animate"
          exit="exit"
          transition={{ duration: 0.2 }}
        >
          {step === 0 && <WelcomeStep />}
          {step === 1 && <SttSetupStep />}
          {step === 2 && <LlmSetupStep />}
          {step === 3 && <DoneStep />}
        </motion.div>
      </AnimatePresence>
    </OnboardingLayout>
  )
}
