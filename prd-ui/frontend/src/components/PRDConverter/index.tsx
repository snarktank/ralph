import { useState, useEffect } from 'react';
import ProjectSelector from '../ProjectSelector';
import PRDSelector from './PRDSelector';
import JSONPreview from './JSONPreview';
import SaveJSONStep from './SaveJSONStep';
import { useProject } from '../../hooks/useProject';
import type { PRDJSON } from '../../types/prd';
import './PRDConverter.css';

type Step = 'project' | 'select' | 'preview' | 'save';

export default function PRDConverter() {
  const { projectPath: hookProjectPath, isValid } = useProject();
  const [step, setStep] = useState<Step>('project');
  const [savedProjectPath, setSavedProjectPath] = useState<string>('');
  const [selectedPRD, setSelectedPRD] = useState<string>('');
  const [prdContent, setPrdContent] = useState('');
  const [jsonData, setJsonData] = useState<PRDJSON | null>(null);
  const [projectName, setProjectName] = useState('');

  // Update saved project path when hook project path changes and is valid
  useEffect(() => {
    if (hookProjectPath && isValid && !savedProjectPath) {
      setSavedProjectPath(hookProjectPath);
    }
  }, [hookProjectPath, isValid, savedProjectPath]);

  const handleProjectSelected = (path?: string) => {
    // Save the project path from the callback (most reliable)
    const pathToSave = path || hookProjectPath;
    if (pathToSave) {
      setSavedProjectPath(pathToSave);
    } else {
      console.error('No project path available when continuing to select step');
      return;
    }
    setStep('select');
  };

  const handlePRDSelected = (filename: string, content: string) => {
    setSelectedPRD(filename);
    setPrdContent(content);
    setStep('preview');
  };

  const handleJSONGenerated = (json: PRDJSON) => {
    setJsonData(json);
    setStep('save');
  };

  const handleBack = () => {
    if (step === 'save') setStep('preview');
    else if (step === 'preview') setStep('select');
    else if (step === 'select') setStep('project');
  };

  return (
    <div className="prd-converter">
      <div className="step-indicator">
        <div className={step === 'project' ? 'active' : step !== 'project' ? 'completed' : ''}>
          1. Project
        </div>
        <div className={step === 'select' ? 'active' : step === 'preview' || step === 'save' ? 'completed' : ''}>
          2. Select PRD
        </div>
        <div className={step === 'preview' ? 'active' : step === 'save' ? 'completed' : ''}>
          3. Preview JSON
        </div>
        <div className={step === 'save' ? 'active' : ''}>
          4. Save
        </div>
      </div>

      {step === 'project' && (
        <ProjectSelector onProjectSelected={handleProjectSelected} />
      )}

      {step === 'select' && (
        <PRDSelector
          projectPath={savedProjectPath || hookProjectPath}
          onPRDSelected={handlePRDSelected}
          onBack={handleBack}
        />
      )}

      {step === 'preview' && (
        <JSONPreview
          projectPath={savedProjectPath || hookProjectPath}
          prdContent={prdContent}
          selectedPRD={selectedPRD}
          onJSONGenerated={handleJSONGenerated}
          onBack={handleBack}
          onProjectNameChange={setProjectName}
        />
      )}

      {step === 'save' && jsonData && (
        <SaveJSONStep
          projectPath={savedProjectPath || hookProjectPath}
          jsonData={jsonData}
          projectName={projectName}
          onBack={handleBack}
        />
      )}
    </div>
  );
}
