import { useState, useEffect } from 'react';
import { prdApi } from '../../services/api';
import type { Question } from '../../types/prd';
import './QuestionsStep.css';

interface QuestionsStepProps {
  featureDescription: string;
  onSubmit: (questions: Question[], answers: Record<string, string>) => void;
  onBack: () => void;
}

export default function QuestionsStep({ featureDescription, onSubmit, onBack }: QuestionsStepProps) {
  const [questions, setQuestions] = useState<Question[]>([]);
  const [answers, setAnswers] = useState<Record<string, string>>({});
  const [customAnswers, setCustomAnswers] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadQuestions();
  }, [featureDescription]);

  const loadQuestions = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await prdApi.generateQuestions(featureDescription);
      setQuestions(response.data.questions || []);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to generate questions');
    } finally {
      setLoading(false);
    }
  };

  const isOtherOption = (option: string): boolean => {
    const lower = option.toLowerCase();
    return lower.includes('other') && (lower.includes('please specify') || lower.includes('specify'));
  };

  const handleAnswerChange = (questionId: string, answer: string) => {
    const isOther = isOtherOption(answer);
    if (isOther) {
      // When "Other" is selected, keep the option text but prepare for custom input
      setAnswers(prev => ({ ...prev, [questionId]: answer }));
      // Initialize custom answer if not already set
      setCustomAnswers(prev => {
        if (!prev[questionId]) {
          return { ...prev, [questionId]: '' };
        }
        return prev;
      });
    } else {
      // Regular option selected, clear custom answer
      setAnswers(prev => ({ ...prev, [questionId]: answer }));
      setCustomAnswers(prev => {
        const updated = { ...prev };
        delete updated[questionId];
        return updated;
      });
    }
  };

  const handleCustomAnswerChange = (questionId: string, customAnswer: string) => {
    setCustomAnswers(prev => ({ ...prev, [questionId]: customAnswer }));
    // Update the main answer with the custom text
    setAnswers(prev => ({ ...prev, [questionId]: customAnswer }));
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    // Merge custom answers into main answers
    const finalAnswers = { ...answers };
    Object.keys(customAnswers).forEach(questionId => {
      if (customAnswers[questionId]) {
        finalAnswers[questionId] = customAnswers[questionId];
      }
    });
    onSubmit(questions, finalAnswers);
  };

  if (loading) {
    return (
      <div className="questions-step">
        <div className="card">
          <div>Generating questions...</div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="questions-step">
        <div className="card">
          <div className="error-message">{error}</div>
          <button onClick={loadQuestions}>Retry</button>
        </div>
      </div>
    );
  }

  return (
    <div className="questions-step">
      <div className="card">
        <h2>Answer Clarifying Questions</h2>
        <form onSubmit={handleSubmit}>
          {questions.map((question) => (
            <div key={question.id} className="question-group">
              <label>{question.text}</label>
              <div className="options">
                {question.options.map((option, index) => {
                  const optionId = `${question.id}-${index}`;
                  const letter = String.fromCharCode(65 + index); // A, B, C, D...
                  const isOther = isOtherOption(option);
                  const currentAnswer = answers[question.id] || '';
                  // Check if this option is selected:
                  // 1. Direct match with option text
                  // 2. For "Other" option: if answer exists and is not in the options list (custom answer)
                  const isSelected = currentAnswer === option || 
                    (isOther && currentAnswer && !question.options.includes(currentAnswer));
                  return (
                    <div key={optionId}>
                      <label className="option">
                        <input
                          type="radio"
                          name={question.id}
                          value={option}
                          checked={!!isSelected}
                          onChange={(e) => handleAnswerChange(question.id, e.target.value)}
                        />
                        <span className="option-letter">{letter}.</span>
                        <span className="option-text">{option}</span>
                      </label>
                      {isOther && isSelected && (
                        <div className="custom-answer-field">
                          <label htmlFor={`${question.id}-custom`}>Please specify:</label>
                          <textarea
                            id={`${question.id}-custom`}
                            value={customAnswers[question.id] || ''}
                            onChange={(e) => handleCustomAnswerChange(question.id, e.target.value)}
                            placeholder="Enter your custom answer..."
                            rows={3}
                            required
                          />
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
            </div>
          ))}

          <div className="form-actions">
            <button type="button" onClick={onBack}>Back</button>
            <button type="submit">Next: Review PRD</button>
          </div>
        </form>
      </div>
    </div>
  );
}
