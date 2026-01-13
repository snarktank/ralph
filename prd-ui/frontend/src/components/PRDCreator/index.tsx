import { useState, useEffect } from 'react';
import ProjectSelector from '../ProjectSelector';
import FeatureInput from './FeatureInput';
import QuestionsStep from './QuestionsStep';
import PRDEditor from './PRDEditor';
import SaveStep from './SaveStep';
import { useProject } from '../../hooks/useProject';
import type { Question } from '../../types/prd';
import './PRDCreator.css';

type Step = 'project' | 'feature' | 'questions' | 'editor' | 'save';

export default function PRDCreator() {
  const { projectPath: hookProjectPath, isValid } = useProject();
  const [step, setStep] = useState<Step>('project');
  const [savedProjectPath, setSavedProjectPath] = useState<string>('');
  const [featureDescription, setFeatureDescription] = useState('');
  const [questions, setQuestions] = useState<Question[]>([]);
  const [answers, setAnswers] = useState<Record<string, string>>({});
  const [prdContent, setPrdContent] = useState('');
  const [featureName, setFeatureName] = useState('');
  const [projectName, setProjectName] = useState('');

  // Update saved project path when hook project path changes and is valid
  // This ensures we have the path even if the callback doesn't provide it
  useEffect(() => {
    if (hookProjectPath && isValid && !savedProjectPath) {
      setSavedProjectPath(hookProjectPath);
    }
  }, [hookProjectPath, isValid, savedProjectPath]);

  const handleProjectSelected = (path?: string) => {
    // Save the project path from the callback (most reliable)
    // The callback provides the path from ProjectSelector's hook instance
    // Fallback to hook path if callback doesn't provide it
    const pathToSave = path || hookProjectPath;
    if (pathToSave) {
      setSavedProjectPath(pathToSave);
    } else {
      // If no path available, don't proceed - this shouldn't happen if validation worked
      console.error('No project path available when continuing to feature step');
      return;
    }
    setStep('feature');
  };

  const handleFeatureSubmit = (description: string, name: string, projName: string) => {
    setFeatureDescription(description);
    setFeatureName(name);
    setProjectName(projName);
    setStep('questions');
  };

  const handleQuestionsSubmit = (qs: Question[], ans: Record<string, string>) => {
    setQuestions(qs);
    setAnswers(ans);
    setStep('editor');
  };

  const handleEditorSubmit = (content: string) => {
    setPrdContent(content);
    setStep('save');
  };

  const handleBack = () => {
    if (step === 'save') setStep('editor');
    else if (step === 'editor') setStep('questions');
    else if (step === 'questions') setStep('feature');
    else if (step === 'feature') setStep('project');
  };

  return (
    <div className="prd-creator">
      <div className="step-indicator">
        <div className={step === 'project' ? 'active' : step !== 'project' ? 'completed' : ''}>
          1. Project
        </div>
        <div className={step === 'feature' ? 'active' : step === 'questions' || step === 'editor' || step === 'save' ? 'completed' : ''}>
          2. Feature
        </div>
        <div className={step === 'questions' ? 'active' : step === 'editor' || step === 'save' ? 'completed' : ''}>
          3. Questions
        </div>
        <div className={step === 'editor' ? 'active' : step === 'save' ? 'completed' : ''}>
          4. Review
        </div>
        <div className={step === 'save' ? 'active' : ''}>
          5. Save
        </div>
      </div>

      {step === 'project' && (
        <ProjectSelector onProjectSelected={handleProjectSelected} />
      )}

      {step === 'feature' && (
        <FeatureInput
          onSubmit={handleFeatureSubmit}
          onBack={handleBack}
        />
      )}

      {step === 'questions' && (
        <QuestionsStep
          featureDescription={featureDescription}
          onSubmit={handleQuestionsSubmit}
          onBack={handleBack}
        />
      )}

      {step === 'editor' && (
        <PRDEditor
          featureDescription={featureDescription}
          answers={answers}
          questions={questions}
          onSubmit={handleEditorSubmit}
          onBack={handleBack}
        />
      )}

      {step === 'save' && (
        <SaveStep
          projectPath={savedProjectPath || hookProjectPath}
          featureName={featureName}
          prdContent={prdContent}
          projectName={projectName}
          onBack={handleBack}
        />
      )}
    </div>
  );
}
